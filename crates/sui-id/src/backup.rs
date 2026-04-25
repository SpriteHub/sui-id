//! Backup and restore helpers.
//!
//! `backup` produces a tarball containing two files:
//!
//!   * `sui-id.sqlite` — a SQLite-consistent snapshot of the database,
//!     produced via `VACUUM INTO`. Safe to take while sui-id is running.
//!   * `sui-id.key`    — a verbatim copy of the master key file.
//!
//! `restore` is the inverse: it reads such a tarball and writes both files
//! to the paths configured in `[storage]`. To avoid silent overwrites the
//! restore step refuses to clobber an existing database or key file unless
//! `--force` is supplied.
//!
//! The output tarball is created with `0600` permissions because it
//! contains the master key — losing it is equivalent to losing the key.

use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const ENTRY_DB: &str = "sui-id.sqlite";
const ENTRY_KEY: &str = "sui-id.key";

/// Produce a backup tarball at `dest`.
pub fn run_backup(cfg: &Config, dest: &Path) -> Result<()> {
    if dest.exists() {
        bail!("refusing to overwrite existing file {}", dest.display());
    }

    // Step 1: produce a consistent SQLite snapshot via `VACUUM INTO`. We
    // open the database without going through `Database::open` because we
    // do not need migrations or the master key here — `VACUUM INTO` is a
    // pure SQLite operation and the encrypted columns are bytes either
    // way.
    if !cfg.storage.db_path.exists() {
        bail!(
            "configured database does not exist at {}",
            cfg.storage.db_path.display()
        );
    }
    if !cfg.storage.key_file.exists() {
        bail!(
            "configured key file does not exist at {}",
            cfg.storage.key_file.display()
        );
    }

    let snapshot_dir = tempfile_dir()?;
    let snapshot_path = snapshot_dir.join(ENTRY_DB);

    {
        let conn = rusqlite::Connection::open(&cfg.storage.db_path)
            .context("opening source database for snapshot")?;
        // VACUUM INTO produces a fully consistent copy at the given path.
        // The destination must not exist (rusqlite will error otherwise).
        let target = snapshot_path
            .to_str()
            .context("snapshot path must be valid UTF-8")?;
        // Quote the path with single quotes per SQLite literal syntax;
        // single quotes inside the path are doubled.
        let quoted = target.replace('\'', "''");
        conn.execute_batch(&format!("VACUUM INTO '{quoted}'"))
            .context("VACUUM INTO failed")?;
    }

    // Step 2: read both artefacts.
    let db_bytes = std::fs::read(&snapshot_path)
        .context("reading database snapshot")?;
    let key_bytes = std::fs::read(&cfg.storage.key_file)
        .context("reading master key file")?;

    // Step 3: build a minimal tarball at `dest` with mode 0600.
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    let mut out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(dest)
        .with_context(|| format!("creating backup file {}", dest.display()))?;

    write_tar_entry(&mut out, ENTRY_DB, &db_bytes)?;
    write_tar_entry(&mut out, ENTRY_KEY, &key_bytes)?;
    write_tar_terminator(&mut out)?;
    out.sync_all().ok();

    // Best-effort: clean up the snapshot file.
    let _ = std::fs::remove_file(&snapshot_path);
    let _ = std::fs::remove_dir(&snapshot_dir);

    Ok(())
}

/// Restore a backup tarball into the configured storage paths.
pub fn run_restore(cfg: &Config, src: &Path, force: bool) -> Result<()> {
    if !src.exists() {
        bail!("backup file {} does not exist", src.display());
    }
    let bytes = std::fs::read(src).with_context(|| format!("reading {}", src.display()))?;
    let entries = read_tar(&bytes)?;
    let db_bytes = entries
        .iter()
        .find(|(name, _)| name == ENTRY_DB)
        .map(|(_, b)| b)
        .with_context(|| format!("backup is missing {ENTRY_DB} entry"))?;
    let key_bytes = entries
        .iter()
        .find(|(name, _)| name == ENTRY_KEY)
        .map(|(_, b)| b)
        .with_context(|| format!("backup is missing {ENTRY_KEY} entry"))?;

    if !force {
        if cfg.storage.db_path.exists() {
            bail!(
                "refusing to overwrite existing database at {} (pass --force to override)",
                cfg.storage.db_path.display()
            );
        }
        if cfg.storage.key_file.exists() {
            bail!(
                "refusing to overwrite existing key file at {} (pass --force to override)",
                cfg.storage.key_file.display()
            );
        }
    }

    if let Some(parent) = cfg.storage.db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    if let Some(parent) = cfg.storage.key_file.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    write_atomic(&cfg.storage.db_path, db_bytes, 0o600)?;
    write_atomic(&cfg.storage.key_file, key_bytes, 0o600)?;
    Ok(())
}

fn write_atomic(target: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    let tmp = target.with_extension("restoring");
    if tmp.exists() {
        std::fs::remove_file(&tmp).ok();
    }
    {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(mode)
            .open(&tmp)
            .with_context(|| format!("creating temp file {}", tmp.display()))?;
        f.write_all(bytes)?;
        f.sync_all().ok();
    }
    std::fs::rename(&tmp, target)
        .with_context(|| format!("renaming temp file into {}", target.display()))?;
    Ok(())
}

fn tempfile_dir() -> Result<PathBuf> {
    // Use a per-process directory under the system temp dir so concurrent
    // backups (rare, but possible from a cron) do not collide.
    let base = std::env::temp_dir();
    let unique = format!("sui-id-backup-{}", std::process::id());
    let dir = base.join(unique);
    std::fs::create_dir_all(&dir).context("creating temp dir for snapshot")?;
    Ok(dir)
}

// ---------- minimal POSIX ustar tar writer / reader ----------------------
// The `tar` crate is a perfectly good dependency, but for two files we can
// stay zero-dep and keep the audit surface small. Format reference:
// https://www.gnu.org/software/tar/manual/html_node/Standard.html

const BLOCK: usize = 512;

fn write_tar_entry(out: &mut File, name: &str, bytes: &[u8]) -> Result<()> {
    if name.len() >= 100 {
        bail!("tar entry name too long: {name}");
    }
    let mut header = [0u8; BLOCK];
    // name (offset 0, 100 bytes)
    header[..name.len()].copy_from_slice(name.as_bytes());
    // mode (100, 8 bytes, octal ASCII, NUL-terminated). 0600.
    write_octal(&mut header[100..108], 0o600);
    // uid, gid (108, 8 bytes each). 0.
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    // size (124, 12 bytes octal)
    write_octal(&mut header[124..136], bytes.len() as u64);
    // mtime (136, 12 bytes octal) — 0 is acceptable for an archive.
    write_octal(&mut header[136..148], 0);
    // chksum (148, 8 bytes) — fill with spaces for the checksum
    // computation, then overwrite with the result.
    for b in &mut header[148..156] {
        *b = b' ';
    }
    // typeflag (156, 1 byte) — '0' = regular file.
    header[156] = b'0';
    // linkname (157, 100 bytes) zero-filled.
    // magic (257, 6) "ustar\0"
    header[257..263].copy_from_slice(b"ustar\0");
    // version (263, 2)
    header[263..265].copy_from_slice(b"00");
    // uname/gname (265, 32 each) — leave empty.
    // devmajor/devminor (329, 8 each) — 0.
    write_octal(&mut header[329..337], 0);
    write_octal(&mut header[337..345], 0);

    let chksum: u32 = header.iter().map(|&b| b as u32).sum();
    // 6 octal digits, NUL, space.
    let s = format!("{chksum:06o}\0 ");
    let bytes_chk = s.as_bytes();
    header[148..148 + bytes_chk.len()].copy_from_slice(bytes_chk);

    out.write_all(&header)?;
    out.write_all(bytes)?;
    let pad = (BLOCK - (bytes.len() % BLOCK)) % BLOCK;
    if pad > 0 {
        out.write_all(&vec![0u8; pad])?;
    }
    Ok(())
}

fn write_tar_terminator(out: &mut File) -> Result<()> {
    out.write_all(&[0u8; BLOCK * 2])?;
    Ok(())
}

fn write_octal(buf: &mut [u8], mut value: u64) {
    let n = buf.len();
    // We write `n-1` octal digits, NUL-terminated.
    for i in (0..n - 1).rev() {
        buf[i] = b'0' + (value & 0o7) as u8;
        value >>= 3;
    }
    buf[n - 1] = 0;
}

fn read_tar(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let mut out = Vec::new();
    let mut idx = 0;
    while idx + BLOCK <= bytes.len() {
        let header = &bytes[idx..idx + BLOCK];
        if header.iter().all(|&b| b == 0) {
            break;
        }
        let name_end = header[..100].iter().position(|&b| b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..name_end])
            .context("tar entry name is not UTF-8")?
            .to_owned();
        let size = read_octal(&header[124..136])?;
        idx += BLOCK;
        if idx + (size as usize) > bytes.len() {
            bail!("truncated tar entry for {name}");
        }
        let body = bytes[idx..idx + size as usize].to_vec();
        out.push((name, body));
        // Advance past the rounded-up data area.
        let padded = ((size as usize) + BLOCK - 1) / BLOCK * BLOCK;
        idx += padded;
    }
    if out.is_empty() {
        bail!("tar archive contains no entries");
    }
    Ok(out)
}

fn read_octal(buf: &[u8]) -> Result<u64> {
    let mut v = 0u64;
    for &b in buf {
        if b == 0 || b == b' ' {
            break;
        }
        if !(b'0'..=b'7').contains(&b) {
            bail!("invalid octal digit in tar header");
        }
        v = v * 8 + (b - b'0') as u64;
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::TempDir;

    fn fake_files(dir: &Path) -> (PathBuf, PathBuf) {
        let db = dir.join("sui-id.sqlite");
        let key = dir.join("sui-id.key");
        // For the round-trip test we don't need a real SQLite file; the
        // tar pipe doesn't care. The end-to-end backup() function does
        // need a real SQLite file, exercised separately.
        std::fs::write(&db, b"sqlite-fake-bytes").unwrap();
        std::fs::write(&key, b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
        (db, key)
    }

    #[test]
    fn tar_round_trip_two_entries() {
        let tmp = TempDir::new().expect("tempdir");
        let dest = tmp.path().join("out.tar");
        {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&dest)
                .unwrap();
            write_tar_entry(&mut f, "a", b"hello").unwrap();
            write_tar_entry(&mut f, "b", b"world!!!").unwrap();
            write_tar_terminator(&mut f).unwrap();
        }
        let mut bytes = Vec::new();
        File::open(&dest).unwrap().read_to_end(&mut bytes).unwrap();
        let entries = read_tar(&bytes).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "a");
        assert_eq!(entries[0].1, b"hello");
        assert_eq!(entries[1].0, "b");
        assert_eq!(entries[1].1, b"world!!!");
    }

    #[test]
    fn restore_refuses_to_overwrite_without_force() {
        let tmp = TempDir::new().expect("tempdir");
        let (db, key) = fake_files(tmp.path());
        let cfg = Config {
            server: crate::config::ServerConfig {
                listen_addr: "127.0.0.1:0".into(),
                issuer: "https://x".into(),
                cookie_secure: false,
                trusted_proxies: Vec::new(),
            },
            storage: crate::config::StorageConfig {
                db_path: db.clone(),
                key_file: key.clone(),
            },
            tokens: crate::config::TokensConfig::default(),
            log: crate::config::LogConfig::default(),
        };
        let backup_path = tmp.path().join("backup.tar");
        // Build a backup tar by hand — bypass run_backup since fake_files
        // didn't create a real SQLite file.
        {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&backup_path)
                .unwrap();
            write_tar_entry(&mut f, ENTRY_DB, b"db-bytes").unwrap();
            write_tar_entry(&mut f, ENTRY_KEY, b"key-bytes").unwrap();
            write_tar_terminator(&mut f).unwrap();
        }
        // db & key already exist, so restore must refuse.
        let r = run_restore(&cfg, &backup_path, false);
        assert!(r.is_err(), "expected refusal to overwrite without --force");
        // With --force, it succeeds.
        run_restore(&cfg, &backup_path, true).expect("force restore");
        assert_eq!(std::fs::read(&db).unwrap(), b"db-bytes");
        assert_eq!(std::fs::read(&key).unwrap(), b"key-bytes");
    }

    #[test]
    fn restore_creates_files_when_destinations_dont_exist() {
        let tmp = TempDir::new().expect("tempdir");
        let cfg = Config {
            server: crate::config::ServerConfig {
                listen_addr: "127.0.0.1:0".into(),
                issuer: "https://x".into(),
                cookie_secure: false,
                trusted_proxies: Vec::new(),
            },
            storage: crate::config::StorageConfig {
                db_path: tmp.path().join("subdir").join("sui-id.sqlite"),
                key_file: tmp.path().join("subdir").join("sui-id.key"),
            },
            tokens: crate::config::TokensConfig::default(),
            log: crate::config::LogConfig::default(),
        };
        let backup_path = tmp.path().join("backup.tar");
        {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&backup_path)
                .unwrap();
            write_tar_entry(&mut f, ENTRY_DB, b"db-bytes").unwrap();
            write_tar_entry(&mut f, ENTRY_KEY, b"key-bytes").unwrap();
            write_tar_terminator(&mut f).unwrap();
        }
        run_restore(&cfg, &backup_path, false).expect("restore");
        assert!(cfg.storage.db_path.exists());
        assert!(cfg.storage.key_file.exists());
    }

    #[test]
    fn run_backup_round_trip_via_real_sqlite() {
        let tmp = TempDir::new().expect("tempdir");
        let db = tmp.path().join("source.sqlite");
        let key = tmp.path().join("source.key");
        // Real SQLite file.
        {
            let conn = rusqlite::Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE t (k TEXT PRIMARY KEY); INSERT INTO t VALUES ('hello');",
            )
            .unwrap();
        }
        std::fs::write(&key, b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
        let cfg = Config {
            server: crate::config::ServerConfig {
                listen_addr: "127.0.0.1:0".into(),
                issuer: "https://x".into(),
                cookie_secure: false,
                trusted_proxies: Vec::new(),
            },
            storage: crate::config::StorageConfig {
                db_path: db.clone(),
                key_file: key.clone(),
            },
            tokens: crate::config::TokensConfig::default(),
            log: crate::config::LogConfig::default(),
        };
        let dest = tmp.path().join("backup.tar");
        run_backup(&cfg, &dest).expect("backup");
        assert!(dest.exists());
        // Verify mode 0600.
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        // Restore into a fresh location and check the SQLite file is queryable.
        let cfg2 = Config {
            server: cfg.server.clone(),
            storage: crate::config::StorageConfig {
                db_path: tmp.path().join("restored.sqlite"),
                key_file: tmp.path().join("restored.key"),
            },
            tokens: cfg.tokens.clone(),
            log: cfg.log.clone(),
        };
        run_restore(&cfg2, &dest, false).expect("restore");
        let conn = rusqlite::Connection::open(&cfg2.storage.db_path).unwrap();
        let v: String = conn
            .query_row("SELECT k FROM t LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "hello");
        // Key file restored byte-for-byte.
        let restored_key = std::fs::read(&cfg2.storage.key_file).unwrap();
        assert_eq!(restored_key, b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    }
}
