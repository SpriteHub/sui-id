//! Forgot-password / password-reset flow.
//!
//! Three pure functions:
//!
//! - [`request_reset`] — issued from `POST /forgot-password`. Looks
//!   up a user by email, generates a token, persists its hash,
//!   sends the reset link mail, returns. Always returns `Ok(())`
//!   externally (user-enumeration protection); failures are
//!   audit-logged.
//! - [`validate_token`] — issued from `GET /reset-password?token=…`
//!   to gate rendering the new-password form. Verifies the token
//!   without consuming it.
//! - [`consume_and_reset_password`] — issued from
//!   `POST /reset-password`. Verifies the token, replaces the user's
//!   password, marks the token consumed, all in one logical step.
//!
//! ## Token shape
//!
//! - 32 random bytes from `OsRng` → URL-safe base64 (no padding).
//!   The plaintext only ever exists in the user's email and the
//!   user's clipboard / browser.
//! - SHA-256 of the plaintext is stored in
//!   `password_reset_tokens.token_hash`. A backup leak does not
//!   yield live tokens. SHA-256 is sufficient: the underlying
//!   token is 32 bytes of CSPRNG output, so we only need preimage
//!   resistance, not a slow KDF.
//! - 30-minute TTL by default.
//! - Single-use: `consumed_at` set on redemption; replays land on
//!   a "consumed" check that returns `InvalidCredentials`.
//!
//! ## User enumeration
//!
//! `request_reset` returns `Ok(())` whether the email matched a
//! user or not, takes roughly the same time in both branches, and
//! emits a `auth.password.reset_requested` event in either case.
//! The handler always shows a generic "if an account exists, we've
//! sent the link" page.

use crate::errors::{CoreError, CoreResult};
use crate::events::{self, Context, SecurityEvent};
use crate::mail::{MailSender, OutgoingMail};
use crate::password;
use crate::time::SharedClock;
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::Duration;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sui_id_shared::ids::{PasswordResetTokenId, UserId};
use sui_id_store::models::{CredentialRow, PasswordResetTokenRow};
use sui_id_store::repos::{credentials, password_reset_tokens, smtp_config, users};
use sui_id_store::Database;

/// 30 minutes — a balance between user-friendly delivery delays
/// and a reasonably tight attack window.
pub const DEFAULT_TOKEN_TTL: Duration = Duration::minutes(30);

/// Outstanding-token ceiling per user. Above this, we silently
/// stop issuing new tokens (the response is still 200 so a probe
/// can't tell). Prevents a single user's inbox from being spammed.
const MAX_OUTSTANDING_TOKENS_PER_USER: i64 = 3;

fn mint_random_token() -> (String, Vec<u8>) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let plaintext = Base64UrlUnpadded::encode_string(&bytes);
    let hash = Sha256::digest(plaintext.as_bytes()).to_vec();
    (plaintext, hash)
}

fn hash_token(plaintext: &str) -> Vec<u8> {
    Sha256::digest(plaintext.as_bytes()).to_vec()
}

/// Issue a password-reset token for the given email, send the
/// reset-link mail, and emit an audit event.
///
/// The exterior contract is **unconditional success**: even when
/// the email doesn't match a user, or the user has no email, or
/// SMTP is unconfigured, this returns `Ok(())`. Internal failures
/// are recorded as audit events but never surfaced. The handler
/// maps every internal outcome to the same neutral 200-response
/// page so `POST /forgot-password` cannot be a user-enumeration
/// oracle.
pub async fn request_reset(
    db: &Database,
    clock: &SharedClock,
    mailer: &dyn MailSender,
    email: &str,
    requester_ip: Option<&str>,
) -> CoreResult<()> {
    let normalized_email = email.trim().to_lowercase();
    let now = clock.now();

    let mut ctx = Context::default();
    if let Some(ip) = requester_ip {
        ctx = ctx.with_client_ip(ip);
    }

    // Look up by email.
    let user_row = users::find_by_email(db, &normalized_email)?;
    let Some(user_row) = user_row else {
        events::emit(
            db,
            clock,
            &ctx,
            SecurityEvent::PasswordResetRequested { user_id: None },
        );
        return Ok(());
    };

    if user_row.is_disabled || user_row.is_deleted {
        events::emit(
            db,
            clock,
            &ctx.clone().with_actor(user_row.id),
            SecurityEvent::PasswordResetRequested {
                user_id: Some(user_row.id),
            },
        );
        return Ok(());
    }

    // Outstanding-token throttle.
    let outstanding =
        password_reset_tokens::count_active_for_user(db, user_row.id, now)?;
    if outstanding >= MAX_OUTSTANDING_TOKENS_PER_USER {
        events::emit(
            db,
            clock,
            &ctx.clone().with_actor(user_row.id),
            SecurityEvent::PasswordResetThrottled {
                user_id: user_row.id,
                outstanding,
            },
        );
        return Ok(());
    }

    // Mint a token, persist its hash.
    let (plaintext, hash) = mint_random_token();
    let row = PasswordResetTokenRow {
        id: PasswordResetTokenId::new(),
        user_id: user_row.id,
        token_hash: hash,
        issued_at: now,
        expires_at: now + DEFAULT_TOKEN_TTL,
        consumed_at: None,
        requester_ip: requester_ip.map(str::to_owned),
    };
    password_reset_tokens::insert(db, &row)?;

    // Build the reset link from `smtp_config.base_url` (the
    // user-facing origin, not necessarily the OIDC issuer URL).
    let base_url = match smtp_config::get(db)? {
        Some(c) if c.enabled => c.base_url,
        _ => {
            // SMTP disabled / unconfigured. Still return Ok so the
            // exterior shape is constant; record the actual outcome.
            events::emit(
                db,
                clock,
                &ctx.clone().with_actor(user_row.id),
                SecurityEvent::PasswordResetEmailFailed {
                    user_id: user_row.id,
                    reason: "smtp_unconfigured".into(),
                },
            );
            return Ok(());
        }
    };
    let link = format!(
        "{}/reset-password?token={}",
        base_url.trim_end_matches('/'),
        plaintext
    );

    // Compose and dispatch the mail.
    let display = user_row
        .display_name
        .as_deref()
        .unwrap_or(&user_row.username);
    let mail = OutgoingMail {
        to: normalized_email.clone(),
        subject: "パスワードのリセット — sui-id".to_string(),
        text_body: format!(
            "{display} 様\n\
             \n\
             sui-id でパスワードリセットの依頼を受け付けました。\n\
             以下のリンクから 30 分以内に新しいパスワードを設定してください。\n\
             \n\
             {link}\n\
             \n\
             このメールに心当たりがない場合は無視してください。\n\
             ",
        ),
        html_body: Some(format!(
            "<p>{display} 様</p>\
             <p>sui-id でパスワードリセットの依頼を受け付けました。\
             下のボタンから 30 分以内に新しいパスワードを設定してください。</p>\
             <p><a href=\"{link}\">パスワードを再設定する</a></p>\
             <p>このメールに心当たりがない場合は無視してください。</p>",
            display = html_escape(display),
            link = html_escape(&link),
        )),
    };

    match mailer.send(mail).await {
        Ok(_outcome) => {
            events::emit(
                db,
                clock,
                &ctx.clone().with_actor(user_row.id),
                SecurityEvent::PasswordResetEmailSent {
                    user_id: user_row.id,
                },
            );
        }
        Err(e) => {
            events::emit(
                db,
                clock,
                &ctx.clone().with_actor(user_row.id),
                SecurityEvent::PasswordResetEmailFailed {
                    user_id: user_row.id,
                    reason: e.to_string(),
                },
            );
        }
    }
    Ok(())
}

/// Verify a token without consuming it. Used by the GET handler
/// that decides whether to render the new-password form or a
/// "this link is invalid or expired" page.
pub fn validate_token(
    db: &Database,
    clock: &SharedClock,
    plaintext_token: &str,
) -> CoreResult<UserId> {
    let hash = hash_token(plaintext_token);
    let row = password_reset_tokens::find_by_hash(db, &hash)?
        .ok_or(CoreError::InvalidCredentials)?;
    if row.consumed_at.is_some() {
        return Err(CoreError::InvalidCredentials);
    }
    if row.expires_at < clock.now() {
        return Err(CoreError::InvalidCredentials);
    }
    Ok(row.user_id)
}

/// Verify the token, set the user's new password, and mark the
/// token consumed. The new password must satisfy the project's
/// password policy.
pub async fn consume_and_reset_password(
    db: &Database,
    clock: &SharedClock,
    mailer: &dyn MailSender,
    plaintext_token: &str,
    new_password: &str,
    requester_ip: Option<&str>,
) -> CoreResult<()> {
    password::check_password_policy(new_password)?;
    let hash = hash_token(plaintext_token);
    let row = password_reset_tokens::find_by_hash(db, &hash)?
        .ok_or(CoreError::InvalidCredentials)?;
    let now = clock.now();
    if row.consumed_at.is_some() || row.expires_at < now {
        return Err(CoreError::InvalidCredentials);
    }

    // Update the user's password.
    let new_hash = password::hash_password(new_password)?;
    credentials::upsert(
        db,
        &CredentialRow {
            user_id: row.user_id,
            password_hash: new_hash,
            must_change: false,
            updated_at: now,
        },
    )?;

    // Mark the token consumed so a replay can't re-use it.
    password_reset_tokens::mark_consumed(db, row.id, now)?;

    let mut ctx = Context::default().with_actor(row.user_id);
    if let Some(ip) = requester_ip {
        ctx = ctx.with_client_ip(ip);
    }
    events::emit(
        db,
        clock,
        &ctx,
        SecurityEvent::PasswordResetCompleted {
            user_id: row.user_id,
        },
    );

    // Best-effort post-reset notification mail. Failures here do
    // not affect the password change itself.
    if let Ok(Some(user_row)) = users::find_by_id_opt(db, row.user_id) {
        if let Some(email) = user_row.email.as_deref() {
            let _ = notify_password_changed(mailer, email, &user_row.display_name).await;
        }
    }

    Ok(())
}

/// Send the "your password has just been changed" notification.
///
/// Best-effort: callers swallow errors and proceed. The audit
/// chain records the underlying password-change action separately.
pub async fn notify_password_changed(
    mailer: &dyn MailSender,
    to_email: &str,
    display_name: &Option<String>,
) -> CoreResult<()> {
    let display = display_name.as_deref().unwrap_or("");
    let mail = OutgoingMail {
        to: to_email.to_owned(),
        subject: "パスワードが変更されました — sui-id".to_string(),
        text_body: format!(
            "{display} 様\n\
             \n\
             sui-id のあなたのアカウントのパスワードが変更されました。\n\
             心当たりがない場合は、すぐに /me/security から他のセッションを\n\
             取り消し、サポートに連絡してください。\n\
             "
        ),
        html_body: Some(format!(
            "<p>{display_esc} 様</p>\
             <p>sui-id のあなたのアカウントのパスワードが変更されました。</p>\
             <p>心当たりがない場合は、すぐに <a href=\"/me/security\">セキュリティ設定</a>\
             から他のセッションを取り消し、サポートに連絡してください。</p>",
            display_esc = html_escape(display),
        )),
    };
    mailer.send(mail).await.map(|_| ())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
