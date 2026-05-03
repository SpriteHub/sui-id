//! Step-up challenge endpoints.
//!
//! Mounted at `/me/security/step-up`. Two routes:
//!
//! - `GET  /me/security/step-up?return_to=<path>` — render the
//!   challenge form (TOTP code or recovery code).
//! - `POST /me/security/step-up` — verify the supplied code,
//!   touch `sessions.last_step_up_at`, redirect back to
//!   `return_to`.
//!
//! ## return_to validation
//!
//! `return_to` is a relative path on this origin. The handler
//! refuses anything else: not URL-shaped query strings (`http:` /
//! `https:` / `//` / `\\`), not protocol-relative URLs, not
//! arbitrary `?return_to=https://attacker.example/`. On any
//! suspicious input we silently fall back to `/me/security` —
//! never bouncing the user off-site after a successful auth is
//! the security guarantee.
//!
//! ## What we don't do
//!
//! - We do *not* offer "remember this device for 30 days". The
//!   freshness window is short (5 minutes) by design: it gates
//!   the *next* sensitive action, not a class of devices. If we
//!   wanted long-lived per-device trust we'd need a per-device
//!   token, an "untrust this device" UI, and a binding policy —
//!   all of which are independent features the simple step-up
//!   model can be extended into later if we have a need.
//! - We do *not* support WebAuthn here yet. The step-up TOTP /
//!   recovery flow is enough to gate sensitive actions for any
//!   account that has TOTP enrolled. WebAuthn-only accounts will
//!   currently see an InvalidCredentials response on POST; the
//!   only WebAuthn-only paths today are passkey-first sign-ins
//!   that *also* register TOTP, which the UI strongly nudges
//!   toward. Adding WebAuthn step-up is a follow-up that lifts
//!   the assertion-flow code out of `webauthn.rs` into a shared
//!   helper.

use crate::errors::HttpError;
use crate::handlers::{enforce_csrf, AppStateExt, SessionContext};
use crate::{csrf, handlers::admin::with_csrf_cookie};
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use sui_id_core::errors::CoreError;
use sui_id_web::{Flash, FlashKind};

/// What we treat as a safe `return_to`: a path-only string that
/// starts with `/`, doesn't double-slash (which a browser may
/// interpret as a protocol-relative URL), and contains no
/// backslash, line break, or NUL. Anything else collapses to the
/// default fallback.
fn sanitise_return_to(raw: &str) -> String {
    if raw.is_empty() {
        return "/me/security".to_owned();
    }
    if !raw.starts_with('/') {
        return "/me/security".to_owned();
    }
    if raw.starts_with("//") || raw.starts_with("/\\") {
        return "/me/security".to_owned();
    }
    for c in raw.chars() {
        if c == '\\' || c == '\n' || c == '\r' || c == '\0' {
            return "/me/security".to_owned();
        }
    }
    raw.to_owned()
}

#[derive(Debug, Deserialize)]
pub struct ReturnToQuery {
    #[serde(default)]
    pub return_to: String,
}

pub async fn get(
    state_ext: AppStateExt,
    _ctx: SessionContext,
    jar: CookieJar,
    Query(q): Query<ReturnToQuery>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let return_to = sanitise_return_to(&q.return_to);
    let token = csrf::ensure_token(&jar);
    let html = sui_id_web::render_step_up(&return_to, token.clone(), None);
    let resp = Html(html).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

#[derive(Debug, Deserialize)]
pub struct StepUpForm {
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
    pub code: String,
    #[serde(default)]
    pub return_to: String,
}

pub async fn post(
    state_ext: AppStateExt,
    ctx: SessionContext,
    jar: CookieJar,
    axum::Form(form): axum::Form<StepUpForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    enforce_csrf(&jar, Some(&form.csrf))?;
    let return_to = sanitise_return_to(&form.return_to);

    match sui_id_core::step_up::verify_totp_code(
        &app.db,
        &app.clock,
        ctx.user_id,
        ctx.session_id,
        &form.code,
    ) {
        Ok(()) => {
            // The CSRF cookie was valid; rotate the in-process
            // token but don't burn a fresh one — the sensitive
            // action that follows will use its own CSRF check.
            Ok(Redirect::to(&return_to).into_response())
        }
        Err(CoreError::InvalidCredentials) => {
            let token = csrf::ensure_token(&jar);
            let flash = Flash {
                kind: FlashKind::Error,
                text: "コードが正しくありません。もう一度入力してください。".into(),
            };
            let html = sui_id_web::render_step_up(&return_to, token.clone(), Some(flash));
            let resp = (axum::http::StatusCode::BAD_REQUEST, Html(html)).into_response();
            Ok(with_csrf_cookie(resp, &app, &token))
        }
        Err(other) => Err(HttpError::html(other)),
    }
}
