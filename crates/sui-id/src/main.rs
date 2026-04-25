//! sui-id binary entry point.
//!
//! Usage:
//!     sui-id [--config PATH]
//!     sui-id backup --to PATH [--config PATH]
//!     sui-id restore --from PATH [--config PATH] [--force]
//!     sui-id --print-sample-config
//!
//! With no `--config`, the program looks for `./sui-id.toml`.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use sui_id::{backup, build_router, config::Config, startup};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("sui-id {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "--print-sample-config") {
        let cfg = Config::sample();
        let s = toml::to_string_pretty(&cfg).context("serializing sample config")?;
        println!("{s}");
        return Ok(());
    }

    // Subcommands. Walk the argv carefully: skip past flags that take a
    // value so we don't treat the value (e.g. the path after `--config`)
    // as the subcommand.
    let subcommand = find_subcommand(&args);

    match subcommand.as_deref() {
        Some("backup") => return run_backup_subcommand(&args),
        Some("restore") => return run_restore_subcommand(&args),
        Some(other) => bail!(
            "unknown subcommand {other:?}. Run `sui-id --help` for usage."
        ),
        None => {} // fall through to `serve`.
    }

    serve(&args).await
}

/// First positional argument that is a real subcommand name, not a flag and
/// not the value of a flag that takes one.
fn find_subcommand(args: &[String]) -> Option<String> {
    const FLAGS_WITH_VALUE: &[&str] = &["--config", "--to", "--from"];
    let mut i = 1; // skip program name
    while i < args.len() {
        let a = &args[i];
        if a.starts_with('-') {
            // `--flag=value` is one token; otherwise it consumes the next arg
            // when it's a value-taking flag.
            if FLAGS_WITH_VALUE.contains(&a.as_str()) {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            return Some(a.clone());
        }
    }
    None
}

async fn serve(args: &[String]) -> Result<()> {
    let config_path = parse_config_path(args).unwrap_or_else(|| PathBuf::from("./sui-id.toml"));
    let cfg = Config::load(&config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))?;

    let startup = startup::prepare(cfg)?;
    let router = build_router(startup.state.clone());

    sui_id::gc::spawn(startup.state.clone());

    let addr: std::net::SocketAddr = startup
        .listen_addr
        .parse()
        .with_context(|| format!("invalid listen_addr {}", startup.listen_addr))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    tracing::info!(%addr, "sui-id listening");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("running server")?;
    Ok(())
}

fn run_backup_subcommand(args: &[String]) -> Result<()> {
    let dest = parse_named_path(args, "--to")
        .context("backup requires --to PATH")?;
    let config_path = parse_config_path(args).unwrap_or_else(|| PathBuf::from("./sui-id.toml"));
    let cfg = Config::load(&config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))?;
    backup::run_backup(&cfg, &dest)?;
    eprintln!(
        "backup written to {} (mode 0600). Store it together with knowledge \
         of the master key, but be aware: this archive contains the key.",
        dest.display()
    );
    Ok(())
}

fn run_restore_subcommand(args: &[String]) -> Result<()> {
    let src = parse_named_path(args, "--from")
        .context("restore requires --from PATH")?;
    let config_path = parse_config_path(args).unwrap_or_else(|| PathBuf::from("./sui-id.toml"));
    let cfg = Config::load(&config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))?;
    let force = args.iter().any(|a| a == "--force");
    backup::run_restore(&cfg, &src, force)?;
    eprintln!(
        "restored from {} into {} and {}",
        src.display(),
        cfg.storage.db_path.display(),
        cfg.storage.key_file.display()
    );
    Ok(())
}

fn parse_config_path(args: &[String]) -> Option<PathBuf> {
    parse_named_path(args, "--config")
}

fn parse_named_path(args: &[String], flag: &str) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == flag {
            return iter.next().map(PathBuf::from);
        }
        let prefix = format!("{flag}=");
        if let Some(rest) = a.strip_prefix(&prefix) {
            return Some(PathBuf::from(rest));
        }
    }
    None
}

fn print_help() {
    println!(
        "sui-id {ver}

Self-hosted OpenID Connect provider.

USAGE:
    sui-id [--config PATH]
    sui-id backup --to PATH [--config PATH]
    sui-id restore --from PATH [--config PATH] [--force]
    sui-id --print-sample-config
    sui-id --version
    sui-id --help

SUBCOMMANDS:
    (none)                   Run the HTTP server.
    backup                   Write a tarball containing a SQLite-consistent
                             snapshot of the database and a copy of the
                             master key file. The output file is created
                             with mode 0600. Treat it like the master key.
    restore                  Restore a backup tarball into the configured
                             storage paths. Refuses to overwrite existing
                             files unless --force is supplied.

OPTIONS:
    --config PATH            Path to the TOML configuration file
                             (default: ./sui-id.toml)
    --to PATH                Output path for `backup`.
    --from PATH              Input path for `restore`.
    --force                  Allow `restore` to overwrite existing files.
    --print-sample-config    Print a sample configuration and exit.
    --version, -V            Print version information and exit.
    --help, -h               Print this help and exit.

ENVIRONMENT:
    SUI_ID_MASTER_KEY        Base64-encoded 32-byte master key.
                             Overrides the key file if set.

DOCUMENTATION:
    See README.md and docs/operators.md for the operator's guide.
",
        ver = env!("CARGO_PKG_VERSION")
    );
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    tracing::info!("graceful shutdown initiated");
}
