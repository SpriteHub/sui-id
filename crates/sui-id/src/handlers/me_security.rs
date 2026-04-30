//! `/me/security` — self-service security overview.
//!
//! This is the *user-facing* counterpart to `/admin/audit` and the
//! per-user controls scattered across `/admin/users/.../*`. Where
//! the admin pages are for someone managing other people's
//! accounts, this page is for someone managing their own:
//! seeing where they're signed in, revoking sessions they don't
//! recognise, and reviewing recent authentication events.
//!
//! Routes (mounted from `router.rs`):
//!
//! - `GET  /me/security`
//! - `POST /me/security/sessions/{id}/revoke`
//! - `POST /me/security/sessions/revoke-all-others`
//!
//! The page does not duplicate MFA enrollment UI; that already
//! exists at `/admin/profile` and is reachable by any authenticated
//! user (the `profile_get` handler uses `CurrentUser`, not
//! `CurrentAdmin`). We just deep-link to it.

use crate::handlers::admin::with_csrf_cookie;
use crate::handlers::{enforce_csrf, AppStateExt, CurrentUser};
use crate::{csrf, errors::HttpError};
use axum::extract::{Form, Path, State};
use axum::http::header::LOCATION;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use std::str::FromStr;
use sui_id_core::errors::CoreError;
use sui_id_shared::ids::SessionId;
use sui_id_store::repos::{audit, sessions, user_totp, users};

const SESSION_COOKIE: &str = "sui_id_session";

/// How many recent activity rows to show. Small enough to keep the
/// page easy to scan; users who want a deeper history have a
/// reason to talk to the operator (and the operator has the full
/// `/admin/audit`).
const RECENT_EVENT_LIMIT: i64 = 30;

#[derive(Debug, Deserialize)]
pub struct CsrfOnlyForm {
    #[serde(rename = "_csrf")]
    pub csrf: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokeAllOthersForm {
    #[serde(rename = "_csrf")]
    pub csrf: String,
    /// The session id of the request itself, posted from a hidden
    /// field. We don't trust it on its own — we cross-check against
    /// the cookie — but having it in the form means the keep-set is
    /// explicit and auditable.
    pub current_session: String,
}

pub async fn page_get(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;

    // The `CurrentUser` extractor has already resolved the cookie,
    // but we need the raw cookie value to identify *which* session
    // is the one issuing this request. There can be more than one
    // active session for the same user, and we want to mark the
    // current row as "current".
    let raw_session = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_owned())
        .ok_or_else(|| HttpError::html(CoreError::Unauthenticated))?;
    let current_session_id = SessionId::from_str(&raw_session)
        .map_err(|_| HttpError::html(CoreError::Unauthenticated))?;

    let user = users::get(&app.db, user_id).map_err(|e| HttpError::html(CoreError::from(e)))?;

    let totp_enabled = user_totp::get(&app.db, user_id)
        .map_err(|e| HttpError::html(CoreError::from(e)))?
        .map(|r| r.enabled)
        .unwrap_or(false);

    let passkey_count = sui_id_core::webauthn::list_for_user(&app.db, user_id)
        .map_err(HttpError::html)?
        .len();

    let session_rows =
        sessions::list_active_for_user(&app.db, user_id).map_err(|e| HttpError::html(e.into()))?;
    let mut sessions_view = Vec::with_capacity(session_rows.len());
    for s in session_rows {
        let auth_methods = describe_auth_methods(&s.auth_methods);
        let is_current = s.id == current_session_id;
        sessions_view.push(sui_id_web::MeSessionDescriptor {
            id: s.id.to_string(),
            created_at: s.created_at,
            expires_at: s.expires_at,
            auth_methods,
            is_current,
        });
    }

    let event_rows = audit::recent_for_user(&app.db, user_id, RECENT_EVENT_LIMIT)
        .map_err(|e| HttpError::html(e.into()))?;
    let events_view: Vec<_> = event_rows
        .into_iter()
        .map(|e| sui_id_web::MeAuditEntry {
            at: e.at,
            action: e.action,
            result: e.result,
            note: e.note,
        })
        .collect();

    let token = csrf::ensure_token(&jar);
    let html = sui_id_web::render_me_security(
        sui_id_web::MeSecurityData {
            username: user.username,
            is_admin: user.is_admin,
            totp_enabled,
            passkey_count,
            current_session_id: current_session_id.to_string(),
            sessions: sessions_view,
            recent_events: events_view,
        },
        flash_from_query(&jar),
        token.clone(),
    );
    let resp = Html(html).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

/// Revoke a single session belonging to the current user.
///
/// The `id` path parameter is parsed and then cross-checked against
/// the database: the row must exist *and* belong to the current
/// user. We don't tell the caller which check failed — both refuse
/// with the same 404-shaped redirect, so there's no oracle for
/// guessing other people's session ids.
pub async fn revoke_one(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<String>,
    jar: CookieJar,
    Form(form): Form<CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    enforce_csrf(&jar, Some(&form.csrf))?;

    let target_id = SessionId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid session id".into())))?;

    // Ownership check: pulling the session row and comparing user_id
    // is the simplest correct way. Skipping this would let a user
    // revoke another user's session by guessing the id.
    let row = match sessions::get(&app.db, target_id) {
        Ok(r) => r,
        Err(sui_id_store::StoreError::NotFound) => {
            // Treat unknown ids the same as foreign ids — both
            // produce a redirect back to the page, no leak.
            return Ok(Redirect::to("/me/security?msg=unknown").into_response());
        }
        Err(e) => return Err(HttpError::html(e.into())),
    };
    if row.user_id != user_id {
        return Ok(Redirect::to("/me/security?msg=unknown").into_response());
    }

    sessions::revoke(&app.db, target_id).map_err(|e| HttpError::html(e.into()))?;

    // If the user just revoked their *own* current session, clear
    // the cookie so the next request is clean. They'll bounce to
    // the login page on the redirect target.
    let raw_session = jar.get(SESSION_COOKIE).map(|c| c.value().to_owned());
    if raw_session.as_deref() == Some(target_id.to_string().as_str()) {
        let mut clear = axum_extra::extract::cookie::Cookie::from(SESSION_COOKIE);
        clear.set_path("/");
        clear.make_removal();
        return Ok((
            jar.remove(clear),
            Redirect::to("/admin/login?msg=session_revoked"),
        )
            .into_response());
    }

    Ok(Redirect::to("/me/security?msg=session_revoked").into_response())
}

/// Revoke every session for the current user *except* the one
/// issuing the request. The "keep" id comes from the cookie, not
/// the form field — the form field is decorative; if it disagrees
/// we honour the cookie.
pub async fn revoke_all_others(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
    Form(form): Form<RevokeAllOthersForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    enforce_csrf(&jar, Some(&form.csrf))?;

    let raw_session = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_owned())
        .ok_or_else(|| HttpError::html(CoreError::Unauthenticated))?;
    let keep = SessionId::from_str(&raw_session)
        .map_err(|_| HttpError::html(CoreError::Unauthenticated))?;

    let n = sessions::revoke_all_for_user_except(&app.db, user_id, keep)
        .map_err(|e| HttpError::html(e.into()))?;

    // Audit: emit one row capturing how many sessions were swept.
    // The action name follows the dotted naming used elsewhere.
    let _ = audit::append(
        &app.db,
        &sui_id_store::models::AuditLogRow {
            at: app.clock.now(),
            actor: Some(user_id),
            action: "auth.sessions.bulk_revoke_self".into(),
            target: Some(user_id.to_string()),
            result: "ok".into(),
            note: Some(format!("revoked {n} other session(s)")),
        },
    );

    let target = if n == 0 {
        "/me/security?msg=no_other_sessions"
    } else {
        "/me/security?msg=others_revoked"
    };
    let mut resp = Response::default();
    *resp.status_mut() = axum::http::StatusCode::SEE_OTHER;
    resp.headers_mut().insert(
        LOCATION,
        target.parse().expect("static header value"),
    );
    Ok(resp)
}

// ---------- helpers ----------

fn describe_auth_methods(methods: &[sui_id_shared::AuthMethod]) -> String {
    use sui_id_shared::AuthMethod;
    if methods.is_empty() {
        return "—".into();
    }
    let parts: Vec<&str> = methods
        .iter()
        .map(|m| match m {
            AuthMethod::Pwd => "password",
            AuthMethod::Totp => "TOTP",
            AuthMethod::RecoveryCode => "recovery code",
            AuthMethod::Webauthn => "passkey",
        })
        .collect();
    parts.join(" + ")
}

/// Translate a `?msg=...` query value into a flash. We deliberately
/// keep the set of recognised messages closed — a free-form
/// query-string value would be reflected XSS.
fn flash_from_query(jar: &CookieJar) -> Option<sui_id_web::Flash> {
    // We don't actually have direct access to the query string from
    // the cookie jar — the caller would have to pass it in. For
    // simplicity in this first cut, we don't surface flashes via
    // query strings here; the redirects above are mostly
    // navigational. A subsequent revision can add a `Query<MsgQ>`
    // extractor and translate codes to localised messages.
    let _ = jar;
    None
}
