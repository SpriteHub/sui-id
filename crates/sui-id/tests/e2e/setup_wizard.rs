//! Three-step setup wizard (v0.20.4).
//!
//! Part of the integration test binary; helpers come from
//! [`super::common`].

#![allow(dead_code)]

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use sui_id::build_router;

use tower::ServiceExt;
use super::common::*;

// ---------- v0.20.4: setup wizard 3-step ----------

#[tokio::test]
async fn setup_welcome_renders_when_uninitialized() {
    let state = test_app();
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/setup")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("welcome");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("sui-id へようこそ"));
    assert!(body.contains(r#"href="/setup/admin""#));
    // No form on the welcome page.
    assert!(!body.contains(r#"action="/setup/admin""#));
    // Step indicator shows step 1 active.
    assert!(body.contains("ようこそ"));
    assert!(body.contains("管理者作成"));
    assert!(body.contains("完了"));
}

#[tokio::test]
async fn setup_welcome_redirects_when_initialized() {
    let state = test_app();
    let _ = complete_setup_and_login(&state).await;
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/setup")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("welcome after init");
    assert!(resp.status().is_redirection());
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/admin/login");
}

#[tokio::test]
async fn setup_admin_form_renders_with_email_and_confirm() {
    let state = test_app();
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/setup/admin")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("admin GET");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains(r#"action="/setup/admin""#));
    assert!(body.contains(r#"name="setup_token""#));
    assert!(body.contains(r#"name="username""#));
    assert!(body.contains(r#"name="email""#));
    assert!(body.contains(r#"name="display_name""#));
    assert!(body.contains(r#"name="password""#));
    assert!(body.contains(r#"name="confirm_password""#));
}

#[tokio::test]
async fn setup_admin_post_creates_admin_with_email_and_redirects_to_done() {
    let state = test_app();
    let body = format!(
        "setup_token={SETUP_TOKEN}\
         &username={USERNAME}\
         &display_name=Alice\
         &email=alice%40example.test\
         &password={PASSWORD}\
         &confirm_password={PASSWORD}"
    );
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/setup/admin")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("admin POST");
    assert!(
        resp.status().is_redirection(),
        "expected redirect, got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/setup/done");
    // Session cookie was set so the "次へ" button on /setup/done
    // lands on /admin already authenticated.
    assert!(extract_set_cookie(resp.headers(), "sui_id_session").is_some());

    // The email was persisted on the user row.
    let row = sui_id_store::repos::users::find_by_username(&state.db, USERNAME)
        .expect("user exists");
    assert_eq!(row.email.as_deref(), Some("alice@example.test"));
}

#[tokio::test]
async fn setup_admin_post_rejects_mismatched_confirm() {
    let state = test_app();
    let body = format!(
        "setup_token={SETUP_TOKEN}\
         &username={USERNAME}\
         &email=\
         &password={PASSWORD}\
         &confirm_password=different-password-here"
    );
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/setup/admin")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("admin POST");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("一致しません"));
    // No user was created.
    let result = sui_id_store::repos::users::find_by_username(&state.db, USERNAME);
    assert!(matches!(result, Err(sui_id_store::StoreError::NotFound)));
}

#[tokio::test]
async fn setup_done_renders_after_initialization() {
    let state = test_app();
    let _ = complete_setup_and_login(&state).await;
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/setup/done")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("done");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("セットアップ完了"));
    assert!(body.contains(r#"href="/admin""#));
}

#[tokio::test]
async fn setup_done_says_not_yet_when_uninitialized() {
    let state = test_app();
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/setup/done")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("done before init");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("完了していません"));
    assert!(body.contains(r#"href="/setup""#));
}

#[tokio::test]
async fn admin_users_create_form_accepts_email() {
    let state = test_app();
    let session = complete_setup_and_login(&state).await;

    // Get the users page to obtain a CSRF token.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/users")
                .header(header::COOKIE, format!("sui_id_session={session}"))
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("users GET");
    let csrf =
        extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf cookie set on users GET");
    let body = read_body(resp.into_body()).await;
    let html = String::from_utf8_lossy(&body);
    // The create form must render an email field.
    assert!(html.contains(r#"name="email""#));

    // Submit the form with an email.
    let new_user_pw = "new-user-password-12345";
    let form = format!(
        "username=bob\
         &display_name=Bob\
         &email=bob%40example.test\
         &password={new_user_pw}\
         &_csrf={csrf}"
    );
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/users")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(
                    header::COOKIE,
                    format!("sui_id_session={session}; sui_id_csrf={csrf}"),
                )
                .body(Body::from(form))
                .expect("req"),
        )
        .await
        .expect("users POST");
    assert!(resp.status().is_redirection() || resp.status() == StatusCode::OK);

    // Verify the user has the email persisted.
    let row = sui_id_store::repos::users::find_by_username(&state.db, "bob")
        .expect("bob exists");
    assert_eq!(row.email.as_deref(), Some("bob@example.test"));
}

