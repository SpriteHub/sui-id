//! Internal domain row types.
//!
//! These mirror the DB schema closely. They are the input/output type for
//! the repository functions in [`crate::repos`]. The distinction from the
//! public API DTOs in `sui-id-shared::api` is deliberate: storage and wire
//! formats evolve independently.

use chrono::{DateTime, Utc};
use sui_id_shared::ids::{ClientId, SessionId, SigningKeyId, UserId};

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: UserId,
    pub username: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub is_disabled: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CredentialRow {
    pub user_id: UserId,
    pub password_hash: String,
    pub must_change: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ClientRow {
    pub id: ClientId,
    pub name: String,
    pub confidential: bool,
    pub secret_hash: Option<String>,
    pub redirect_uris: Vec<String>,
    /// Space-separated list of permitted scope values. Empty string means
    /// "no policy configured" — interpreted as "permit any scope" for
    /// backwards compatibility with rows created before migration 0002.
    pub allowed_scopes: String,
    /// Logout return URIs registered for this client. Independent of
    /// `redirect_uris`. An empty list triggers a fall-back to
    /// `redirect_uris` for backwards compatibility (with a deprecation
    /// log line) — see `sui_id_core::session::resolve_post_logout_uri`.
    pub post_logout_redirect_uris: Vec<String>,
    pub is_disabled: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthorizationCodeRow {
    pub code_hash: String,
    pub client_id: ClientId,
    pub user_id: UserId,
    pub redirect_uri: String,
    pub scope: String,
    pub nonce: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub expires_at: DateTime<Utc>,
    pub consumed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: SessionId,
    pub user_id: UserId,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RefreshTokenRow {
    pub id: String,
    pub token_plain: Option<String>, // populated only at issuance
    pub user_id: UserId,
    pub client_id: ClientId,
    pub scope: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SigningKeyRow {
    pub id: SigningKeyId,
    pub algorithm: String,
    /// Sealed bytes of the private key (XChaCha20-Poly1305 of raw 32 bytes).
    pub private_key_enc: Vec<u8>,
    pub public_key: Vec<u8>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AuditLogRow {
    pub at: DateTime<Utc>,
    pub actor: Option<UserId>,
    pub action: String,
    pub target: Option<String>,
    pub result: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserTotpRow {
    pub user_id: UserId,
    /// Sealed TOTP secret bytes (20 bytes when decrypted).
    pub secret_enc: Vec<u8>,
    pub enabled: bool,
    /// Sealed JSON array of Argon2id hashes (one per single-use recovery
    /// code). `None` if the user never generated recovery codes — which is
    /// a temporary state during initial enrolment.
    pub recovery_codes_enc: Option<Vec<u8>>,
    /// Most recently accepted RFC 6238 time step. Used to reject replays
    /// of the same 6-digit code within its 30-second window.
    pub last_used_step: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct LoginPendingMfaRow {
    pub id: sui_id_shared::ids::PendingMfaId,
    pub user_id: UserId,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
