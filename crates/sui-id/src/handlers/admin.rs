//! Admin panel and login.
//!
//! All admin pages render via Leptos SSR through `sui-id-web`. State
//! transitions go via core use cases.

use crate::errors::HttpError;
use crate::handlers::{
    clear_pending_mfa_cookie, clear_pending_mfa_next_cookie, clear_session_cookie,
    pending_mfa_cookie, pending_mfa_next_cookie, session_cookie, AppStateExt, CurrentAdmin,
    CurrentUser, PENDING_MFA_COOKIE, PENDING_MFA_NEXT_COOKIE, SESSION_COOKIE,
};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use std::str::FromStr;
use sui_id_core::admin::{self as admin_uc, CreateUserSpec};
use sui_id_core::errors::CoreError;
use sui_id_core::session;
use sui_id_shared::api::{
    AuditLogEntryDto, ClientSummary, UserSummary,
};
use sui_id_shared::ids::{ClientId, SessionId, UserId};
use sui_id_store::repos::{audit, clients, state, users};
use sui_id_web::{
    pages::DashboardData, render_audit, render_clients, render_dashboard, render_login,
    render_signing_keys, render_users, Flash, FlashKind,
};

/// Attach a `Set-Cookie` header for the CSRF token to a response.
fn with_csrf_cookie(mut resp: Response, app: &AppState, token: &str) -> Response {
    let cookie = crate::csrf::csrf_cookie(token.to_owned(), app.config.server.cookie_secure);
    if let Ok(v) = HeaderValue::from_str(&cookie.to_string()) {
        resp.headers_mut().append(header::SET_COOKIE, v);
    }
    resp
}

// ---------- login ----------

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub next: String,
}

pub async fn login_get(jar: CookieJar, state_ext: AppStateExt) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    // Already logged in?
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Ok(sid) = SessionId::from_str(cookie.value()) {
            if session::resolve(&app.db, &app.clock, sid).is_ok() {
                return Ok(Redirect::to("/admin").into_response());
            }
        }
    }
    Ok(Html(render_login(None, None)).into_response())
}

pub async fn login_post(
    state_ext: AppStateExt,
    crate::handlers::ClientIp(ip): crate::handlers::ClientIp,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_rate_limit(
        &app.limiters,
        &app.clock,
        crate::handlers::RateLimitKey::Login,
        ip,
        crate::handlers::ErrorAs::Html,
    )?;
    match session::login_with_mfa(&app.db, &app.clock, form.username.trim(), &form.password) {
        Ok(session::LoginOutcome::SessionEstablished(row)) => {
            let cookie = session_cookie(row.id.to_string(), app.config.server.cookie_secure);
            let jar = jar.add(cookie);
            let target = if form.next.starts_with('/') {
                form.next.clone()
            } else {
                "/admin".into()
            };
            Ok((jar, Redirect::to(&target)).into_response())
        }
        Ok(session::LoginOutcome::MfaRequired { pending }) => {
            // Drop the user a short-lived cookie pointing at the
            // pending row, and bounce them into the MFA challenge page.
            let cookie = pending_mfa_cookie(
                pending.id.to_string(),
                app.config.server.cookie_secure,
            );
            let next_cookie = if !form.next.is_empty() {
                Some(pending_mfa_next_cookie(
                    form.next.clone(),
                    app.config.server.cookie_secure,
                ))
            } else {
                None
            };
            let jar = jar.add(cookie);
            let jar = match next_cookie {
                Some(c) => jar.add(c),
                None => jar,
            };
            Ok((jar, Redirect::to("/admin/login/mfa")).into_response())
        }
        Err(_) => {
            let flash = Flash {
                kind: FlashKind::Error,
                text: "Sign-in failed. Check your username and password.".into(),
            };
            let next = if form.next.is_empty() {
                None
            } else {
                Some(form.next)
            };
            Ok(
                (StatusCode::UNAUTHORIZED, Html(render_login(Some(flash), next)))
                    .into_response(),
            )
        }
    }
}

pub async fn mfa_challenge_get(
    state_ext: AppStateExt,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    // Caller must already have a pending-mfa cookie. We don't insist on
    // validating the row here — the POST will reject if it's missing or
    // expired — so a stale visit just shows a generic challenge form.
    let _ = &app;
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(sui_id_web::render_mfa_challenge(None, token.clone())).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

#[derive(Debug, Deserialize)]
pub struct MfaChallengeForm {
    pub code: String,
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn mfa_challenge_post(
    state_ext: AppStateExt,
    crate::handlers::ClientIp(ip): crate::handlers::ClientIp,
    jar: CookieJar,
    Form(form): Form<MfaChallengeForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    // Same rate-limit bucket as password attempts: a user who is past
    // the password step still uses a single login budget.
    crate::handlers::enforce_rate_limit(
        &app.limiters,
        &app.clock,
        crate::handlers::RateLimitKey::Login,
        ip,
        crate::handlers::ErrorAs::Html,
    )?;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let pending_value = match jar.get(PENDING_MFA_COOKIE) {
        Some(c) => c.value().to_owned(),
        None => {
            return Ok(Redirect::to("/admin/login").into_response());
        }
    };
    let pending_id = match pending_value.parse::<sui_id_shared::ids::PendingMfaId>() {
        Ok(id) => id,
        Err(_) => return Ok(Redirect::to("/admin/login").into_response()),
    };
    match sui_id_core::mfa::verify_pending(&app.db, &app.clock, pending_id, &form.code) {
        Ok(session) => {
            let cookie =
                session_cookie(session.id.to_string(), app.config.server.cookie_secure);
            // Compose the redirect target from the optional next cookie.
            let next_target = jar
                .get(PENDING_MFA_NEXT_COOKIE)
                .map(|c| c.value().to_owned())
                .filter(|s| s.starts_with('/'))
                .unwrap_or_else(|| "/admin".into());
            // Audit the MFA success.
            let _ = sui_id_store::repos::audit::append(
                &app.db,
                &sui_id_store::models::AuditLogRow {
                    at: app.clock.now(),
                    actor: Some(session.user_id),
                    action: "auth.mfa.success".into(),
                    target: Some(session.user_id.to_string()),
                    result: "ok".into(),
                    note: None,
                },
            );
            let jar = jar
                .add(cookie)
                .add(clear_pending_mfa_cookie(app.config.server.cookie_secure))
                .add(clear_pending_mfa_next_cookie(app.config.server.cookie_secure));
            Ok((jar, Redirect::to(&next_target)).into_response())
        }
        Err(_) => {
            let flash = Flash {
                kind: FlashKind::Error,
                text: "Verification failed. Try again, or use a recovery code.".into(),
            };
            let _ = sui_id_store::repos::audit::append(
                &app.db,
                &sui_id_store::models::AuditLogRow {
                    at: app.clock.now(),
                    actor: None,
                    action: "auth.mfa.failure".into(),
                    target: None,
                    result: "denied".into(),
                    note: None,
                },
            );
            let token = crate::csrf::ensure_token(&jar);
            let resp = (
                StatusCode::UNAUTHORIZED,
                Html(sui_id_web::render_mfa_challenge(Some(flash), token.clone())),
            )
                .into_response();
            Ok(with_csrf_cookie(resp, &app, &token))
        }
    }
}

pub async fn logout(
    jar: CookieJar,
    state_ext: AppStateExt,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    if let Some(c) = jar.get(SESSION_COOKIE) {
        if let Ok(sid) = SessionId::from_str(c.value()) {
            let _ = session::logout(&app.db, sid);
        }
    }
    let jar = jar.add(clear_session_cookie(app.config.server.cookie_secure));
    Ok((jar, Redirect::to("/admin/login")).into_response())
}

// ---------- dashboard ----------

pub async fn dashboard(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let admin = users::get(&app.db, admin_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let users_n = users::list(&app.db)
        .map(|v| v.len())
        .map_err(|e| HttpError::html(CoreError::from(e)))?;
    let clients_n = clients::list(&app.db)
        .map(|v| v.len())
        .map_err(|e| HttpError::html(CoreError::from(e)))?;
    let data = DashboardData {
        admin_username: admin.username,
        user_count: users_n,
        client_count: clients_n,
        issuer: app.issuer().to_owned(),
    };
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_dashboard(data, None)).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

// ---------- users ----------

pub async fn users_get(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let admin = users::get(&app.db, admin_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let rows = admin_uc::list_users(&app.db, admin_id).map_err(HttpError::html)?;
    let summaries: Vec<UserSummary> = rows
        .into_iter()
        .map(|r| UserSummary {
            id: r.id,
            username: r.username,
            display_name: r.display_name,
            is_admin: r.is_admin,
            is_disabled: r.is_disabled,
            is_deleted: r.is_deleted,
            created_at: r.created_at,
        })
        .collect();
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_users(summaries, None, admin.username, token.clone())).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

#[derive(Debug, Deserialize)]
pub struct CreateUserForm {
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    pub password: String,
    #[serde(default)]
    pub is_admin: Option<String>,
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn users_create(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Form(form): Form<CreateUserForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let display = if form.display_name.trim().is_empty() {
        None
    } else {
        Some(form.display_name.as_str())
    };
    let is_admin = form
        .is_admin
        .as_deref()
        .map(|v| matches!(v, "true" | "on" | "1"))
        .unwrap_or(false);
    admin_uc::create_user(
        &app.db,
        &app.clock,
        admin_id,
        CreateUserSpec {
            username: form.username.trim(),
            password: &form.password,
            display_name: display,
            is_admin,
        },
    )
    .map_err(HttpError::html)?;
    let _ = &app; // hush
    Ok(Redirect::to("/admin/users").into_response())
}

#[derive(Debug, Deserialize)]
pub struct DisableForm {
    pub disabled: String,
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn users_set_disabled(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<DisableForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let target = UserId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid user id".into())))?;
    let value = matches!(form.disabled.as_str(), "true" | "on" | "1");
    admin_uc::set_user_disabled(&app.db, admin_id, target, value).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/users").into_response())
}

/// `_csrf`-only body: confirmation-style POSTs that have no other fields.
#[derive(Debug, Deserialize, Default)]
pub struct CsrfOnlyForm {
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn users_delete(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let target = UserId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid user id".into())))?;
    admin_uc::delete_user(&app.db, admin_id, target).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/users").into_response())
}

// ---------- clients ----------

pub async fn clients_get(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let rows = admin_uc::list_clients(&app.db, admin_id).map_err(HttpError::html)?;
    let summaries: Vec<ClientSummary> = rows
        .into_iter()
        .map(|r| ClientSummary {
            id: r.id,
            name: r.name,
            redirect_uris: r.redirect_uris,
            allowed_scopes: r.allowed_scopes,
            post_logout_redirect_uris: r.post_logout_redirect_uris,
            confidential: r.confidential,
            is_disabled: r.is_disabled,
            is_deleted: r.is_deleted,
            created_at: r.created_at,
        })
        .collect();
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_clients(summaries, None, None, token.clone())).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

#[derive(Debug, Deserialize)]
pub struct CreateClientForm {
    pub name: String,
    pub redirect_uris: String,
    #[serde(default)]
    pub confidential: Option<String>,
    #[serde(default)]
    pub allowed_scopes: String,
    #[serde(default)]
    pub post_logout_redirect_uris: String,
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn clients_create(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Form(form): Form<CreateClientForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let uris: Vec<String> = form
        .redirect_uris
        .lines()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let post_logout_uris: Vec<String> = form
        .post_logout_redirect_uris
        .lines()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let confidential = form
        .confidential
        .as_deref()
        .map(|v| matches!(v, "true" | "on" | "1"))
        .unwrap_or(true);
    // Default policy: openid + profile if the operator left the field
    // blank. Empty-after-trim is also accepted as "permit any" but only
    // when the operator explicitly types whitespace; the form's default
    // value is a sensible policy.
    let raw_scopes = form.allowed_scopes.trim();
    let allowed_scopes = if raw_scopes.is_empty() {
        "openid profile"
    } else {
        raw_scopes
    };
    let created = admin_uc::create_client(
        &app.db,
        &app.clock,
        admin_id,
        sui_id_core::admin::CreateClientSpec {
            name: form.name.trim(),
            redirect_uris: &uris,
            confidential,
            allowed_scopes,
            post_logout_redirect_uris: &post_logout_uris,
        },
    )
    .map_err(HttpError::html)?;

    // Re-list and pass the secret through to the page so it is shown once.
    let rows = admin_uc::list_clients(&app.db, admin_id).map_err(HttpError::html)?;
    let summaries: Vec<ClientSummary> = rows
        .into_iter()
        .map(|r| ClientSummary {
            id: r.id,
            name: r.name,
            redirect_uris: r.redirect_uris,
            allowed_scopes: r.allowed_scopes,
            post_logout_redirect_uris: r.post_logout_redirect_uris,
            confidential: r.confidential,
            is_disabled: r.is_disabled,
            is_deleted: r.is_deleted,
            created_at: r.created_at,
        })
        .collect();

    let secret_payload =
        created.generated_secret.map(|s| (created.row.id.to_string(), s));
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_clients(summaries, None, secret_payload, token.clone())).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

pub async fn clients_set_disabled(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<DisableForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let target = ClientId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid client id".into())))?;
    let value = matches!(form.disabled.as_str(), "true" | "on" | "1");
    admin_uc::set_client_disabled(&app.db, admin_id, target, value).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/clients").into_response())
}

pub async fn clients_delete(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let target = ClientId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid client id".into())))?;
    admin_uc::delete_client(&app.db, admin_id, target).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/clients").into_response())
}

// ---------- audit ----------

pub async fn audit_get(
    state_ext: AppStateExt,
    CurrentAdmin(_): CurrentAdmin,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let entries = audit::recent(&app.db, 200)
        .map_err(|e| HttpError::html(CoreError::from(e)))?;
    let dtos: Vec<AuditLogEntryDto> = entries
        .into_iter()
        .map(|r| AuditLogEntryDto {
            at: r.at,
            actor: r.actor,
            action: r.action,
            target: r.target,
            result: r.result,
            note: r.note,
        })
        .collect();
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_audit(dtos, None)).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

// ---------- signing keys ----------

pub async fn signing_keys_get(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let rows = admin_uc::list_signing_keys(&app.db, admin_id).map_err(HttpError::html)?;
    let summaries: Vec<sui_id_shared::api::SigningKeySummary> = rows
        .into_iter()
        .map(|r| sui_id_shared::api::SigningKeySummary {
            id: r.id,
            algorithm: r.algorithm,
            is_active: r.is_active,
            created_at: r.created_at,
            rotated_at: r.rotated_at,
        })
        .collect();
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(render_signing_keys(summaries, None, token.clone())).into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

pub async fn signing_keys_rotate(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Form(form): Form<CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    admin_uc::rotate_signing_key(&app.db, &app.clock, admin_id).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/signing-keys").into_response())
}

pub async fn signing_keys_delete(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let target = sui_id_shared::ids::SigningKeyId::from_str(&id)
        .map_err(|_| HttpError::html(CoreError::BadRequest("invalid signing key id".into())))?;
    admin_uc::delete_signing_key(&app.db, admin_id, target).map_err(HttpError::html)?;
    Ok(Redirect::to("/admin/signing-keys").into_response())
}

// ---------- profile / MFA enrolment ----------

pub async fn profile_get(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    let user = users::get(&app.db, user_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let mfa_enabled = sui_id_core::mfa::is_mfa_enabled(&app.db, user_id).map_err(HttpError::html)?;
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(sui_id_web::render_profile(
        sui_id_web::ProfileData {
            username: user.username,
            mfa_enabled,
            fresh_recovery_codes: None,
        },
        None,
        token.clone(),
    ))
    .into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

pub async fn profile_mfa_enroll_start(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
    Form(form): Form<crate::handlers::admin::CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let user = users::get(&app.db, user_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let ticket = sui_id_core::mfa::start_enrollment(
        &app.db,
        app.issuer(),
        user_id,
        &user.username,
    )
    .map_err(HttpError::html)?;

    // Render QR as SVG via the qrcode crate.
    let qr_svg = render_qr_svg(&ticket.otpauth_uri);
    let secret_b32 = sui_id_core::totp::base32_encode(&ticket.secret);
    let otpauth_uri = ticket.otpauth_uri;
    // The raw secret bytes drop with `ticket` here. sui_id_core::mfa::
    // start_enrollment keeps no caller-visible copy beyond what it
    // returns in the ticket.
    drop(ticket.secret);

    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(sui_id_web::render_mfa_setup(
        sui_id_web::MfaSetupData {
            otpauth_uri,
            qr_svg,
            secret_b32,
        },
        None,
        token.clone(),
    ))
    .into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

#[derive(Debug, Deserialize)]
pub struct MfaConfirmForm {
    pub code: String,
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
}

pub async fn profile_mfa_enroll_confirm(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
    Form(form): Form<MfaConfirmForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let code: u32 = form
        .code
        .trim()
        .parse()
        .map_err(|_| HttpError::html(CoreError::BadRequest("verification code must be 6 digits".into())))?;
    let codes = sui_id_core::mfa::confirm_enrollment(&app.db, &app.clock, user_id, code)
        .map_err(HttpError::html)?;
    let _ = sui_id_store::repos::audit::append(
        &app.db,
        &sui_id_store::models::AuditLogRow {
            at: app.clock.now(),
            actor: Some(user_id),
            action: "mfa.enable".into(),
            target: Some(user_id.to_string()),
            result: "ok".into(),
            note: None,
        },
    );
    let user = users::get(&app.db, user_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(sui_id_web::render_profile(
        sui_id_web::ProfileData {
            username: user.username,
            mfa_enabled: true,
            fresh_recovery_codes: Some(codes),
        },
        Some(Flash {
            kind: FlashKind::Info,
            text: "Two-factor authentication is now enabled.".into(),
        }),
        token.clone(),
    ))
    .into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

pub async fn profile_mfa_disable(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
    Form(form): Form<crate::handlers::admin::CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    sui_id_core::mfa::disable(&app.db, user_id).map_err(HttpError::html)?;
    let _ = sui_id_store::repos::audit::append(
        &app.db,
        &sui_id_store::models::AuditLogRow {
            at: app.clock.now(),
            actor: Some(user_id),
            action: "mfa.disable".into(),
            target: Some(user_id.to_string()),
            result: "ok".into(),
            note: None,
        },
    );
    Ok(Redirect::to("/admin/profile").into_response())
}

pub async fn profile_mfa_regenerate_recovery(
    state_ext: AppStateExt,
    CurrentUser(user_id): CurrentUser,
    jar: CookieJar,
    Form(form): Form<crate::handlers::admin::CsrfOnlyForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    let codes = sui_id_core::mfa::regenerate_recovery_codes(&app.db, user_id)
        .map_err(HttpError::html)?;
    let _ = sui_id_store::repos::audit::append(
        &app.db,
        &sui_id_store::models::AuditLogRow {
            at: app.clock.now(),
            actor: Some(user_id),
            action: "mfa.recovery_codes_regenerate".into(),
            target: Some(user_id.to_string()),
            result: "ok".into(),
            note: None,
        },
    );
    let user = users::get(&app.db, user_id).map_err(|e| HttpError::html(CoreError::from(e)))?;
    let token = crate::csrf::ensure_token(&jar);
    let resp = Html(sui_id_web::render_profile(
        sui_id_web::ProfileData {
            username: user.username,
            mfa_enabled: true,
            fresh_recovery_codes: Some(codes),
        },
        Some(Flash {
            kind: FlashKind::Info,
            text: "Recovery codes regenerated. Save the new ones - the old ones no longer work.".into(),
        }),
        token.clone(),
    ))
    .into_response();
    Ok(with_csrf_cookie(resp, &app, &token))
}

fn render_qr_svg(uri: &str) -> String {
    use qrcode::render::svg;
    use qrcode::QrCode;
    match QrCode::new(uri.as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(220, 220)
            .quiet_zone(true)
            .build(),
        Err(_) => format!(
            "<p class=\"muted\">QR rendering failed; use the secret key below instead.</p>"
        ),
    }
}

#[allow(dead_code)]
fn _silence_state(_: &CurrentUser) {}
#[allow(dead_code)]
fn _silence_state2() -> Option<bool> {
    let _ = state::is_initialized;
    None
}
