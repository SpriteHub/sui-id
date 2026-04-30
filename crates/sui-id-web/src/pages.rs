//! Page-level components and their public render entry points.
//!
//! Each `render_xxx` function constructs a Leptos view, drives it through
//! the SSR renderer, and returns a complete HTML document. The doctype is
//! prepended manually because `view!{}` only renders the tree it is given.

use crate::layout::Shell;
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use sui_id_shared::api::{AuditLogEntryDto, ClientSummary, UserSummary};

const DOCTYPE: &str = "<!DOCTYPE html>";

/// Severity of a flash banner displayed at the top of a page.
#[derive(Debug, Clone, Copy)]
pub enum FlashKind {
    Info,
    Warn,
    Error,
}

impl FlashKind {
    fn class(self) -> &'static str {
        match self {
            Self::Info => "flash info",
            Self::Warn => "flash warn",
            Self::Error => "flash error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Flash {
    pub kind: FlashKind,
    pub text: String,
}

fn flash_banner(flash: Option<Flash>) -> Option<impl IntoView> {
    flash.map(|f| view! { <div class=f.kind.class() role="status">{f.text}</div> })
}

fn fmt_time(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%d %H:%M UTC").to_string()
}

/// Run a closure inside a fresh reactive Owner and prepend the HTML doctype.
fn render<F, V>(f: F) -> String
where
    F: FnOnce() -> V,
    V: IntoView + 'static,
{
    let owner = Owner::new();
    let body = owner.with(|| f().into_view().to_html());
    let mut out = String::with_capacity(DOCTYPE.len() + body.len());
    out.push_str(DOCTYPE);
    out.push_str(&body);
    out
}

// ---------- setup ----------

pub fn render_setup(flash: Option<Flash>) -> String {
    render(move || {
        view! {
            <Shell title="Setup".to_string() show_nav=false current=None>
                <h2>"Welcome to sui-id."</h2>
                <p class="muted">
                    "This server has not been initialized yet. Create the first administrator below. "
                    "The setup token was printed once on this server's standard error at startup; \
                     paste it here to confirm you control the host."
                </p>
                {flash_banner(flash)}
                <form method="post" action="/setup">
                    <label for="token">"Setup token"</label>
                    <input id="token" name="setup_token" type="password" required=true autocomplete="off" />

                    <label for="username">"Administrator username"</label>
                    <input id="username" name="username" type="text" required=true autocomplete="username" />

                    <label for="display">"Display name (optional)"</label>
                    <input id="display" name="display_name" type="text" autocomplete="name" />

                    <label for="password">"Password (12 characters or more)"</label>
                    <input id="password" name="password" type="password" required=true minlength="12" autocomplete="new-password" />

                    <button type="submit">"Create administrator"</button>
                </form>
            </Shell>
        }
    })
}

// ---------- login ----------

pub fn render_login(flash: Option<Flash>, next: Option<String>) -> String {
    render(move || {
        let next_value = next.clone().unwrap_or_default();
        view! {
            <Shell title="Sign in".to_string() show_nav=false current=None>
                <h2>"Sign in"</h2>
                {flash_banner(flash)}
                <form method="post" action="/admin/login">
                    <input type="hidden" name="next" value=next_value />
                    <label for="username">"Username"</label>
                    <input id="username" name="username" type="text" required=true autocomplete="username" />
                    <label for="password">"Password"</label>
                    <input id="password" name="password" type="password" required=true autocomplete="current-password" />
                    <button type="submit">"Sign in"</button>
                </form>
            </Shell>
        }
    })
}

// ---------- MFA challenge ----------

pub fn render_mfa_challenge(
    flash: Option<Flash>,
    csrf_token: String,
    has_passkey: bool,
) -> String {
    render(move || {
        let csrf_for_totp = csrf_token.clone();
        let csrf_for_pk = csrf_token.clone();
        let passkey_block = if has_passkey {
            view! {
                <hr/>
                <p class="muted">"Or, sign in with a passkey:"</p>
                <form id="passkey-auth-form" method="post" action="/admin/login/webauthn/start">
                    <input type="hidden" name="_csrf" value=csrf_for_pk />
                    <button type="submit">"Sign in with passkey"</button>
                </form>
                <script src="/static/webauthn.js"></script>
            }
            .into_any()
        } else {
            view! { <></> }.into_any()
        };
        view! {
            <Shell title="Verification required".to_string() show_nav=false current=None>
                <h2>"Verification code"</h2>
                {flash_banner(flash)}
                <p class="muted">
                    "Enter the 6-digit code from your authenticator app, or one of \
                     your single-use recovery codes."
                </p>
                <form method="post" action="/admin/login/mfa">
                    <input type="hidden" name="_csrf" value=csrf_for_totp />
                    <label for="code">"Code"</label>
                    <input id="code" name="code" type="text" required=true
                           autocomplete="one-time-code" inputmode="text" autofocus=true />
                    <button type="submit">"Verify"</button>
                </form>
                {passkey_block}
            </Shell>
        }
    })
}

// ---------- profile (MFA settings) ----------

pub struct ProfileData {
    pub username: String,
    /// True if TOTP is set up.
    pub totp_enabled: bool,
    /// Set when the user has just enrolled or regenerated codes.
    /// Displayed exactly once.
    pub fresh_recovery_codes: Option<Vec<String>>,
    /// Registered WebAuthn passkeys for this user.
    pub passkeys: Vec<PasskeyDescriptor>,
}

pub struct PasskeyDescriptor {
    pub id: String,
    pub nickname: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn render_profile(data: ProfileData, flash: Option<Flash>, csrf_token: String) -> String {
    render(move || {
        let ProfileData {
            username,
            totp_enabled,
            fresh_recovery_codes,
            passkeys,
        } = data;
        let csrf_for_disable = csrf_token.clone();
        let csrf_for_regen = csrf_token.clone();
        let csrf_for_enroll = csrf_token.clone();
        let csrf_for_passkey_register = csrf_token.clone();
        let csrf_for_passkey_delete = csrf_token.clone();
        let recovery_block = fresh_recovery_codes.map(|codes| {
            let lis: Vec<_> = codes
                .into_iter()
                .map(|c| view! { <li><span class="code">{c}</span></li> })
                .collect();
            view! {
                <div class="flash warn" role="status">
                    <strong>"Save these recovery codes now - they will not be shown again."</strong>
                    <p class="muted">
                        "Each code is single-use. Store them somewhere safe. \
                         If you lose access to your authenticator app you can sign in by typing one in place of the 6-digit code."
                    </p>
                    <ol>{lis}</ol>
                </div>
            }
        });
        let mfa_section = if totp_enabled {
            view! {
                <p>"Authenticator-app two-factor authentication is "<strong>"enabled"</strong>"."</p>
                <form method="post" action="/admin/profile/mfa/recovery-codes/regenerate" style="display:inline">
                    <input type="hidden" name="_csrf" value=csrf_for_regen />
                    <button type="submit" class="secondary">"Regenerate recovery codes"</button>
                </form>
                " "
                <form method="post" action="/admin/profile/mfa/disable" style="display:inline"
                      onsubmit="return confirm('Disable authenticator-app two-factor authentication?');">
                    <input type="hidden" name="_csrf" value=csrf_for_disable />
                    <button type="submit" class="danger">"Disable TOTP"</button>
                </form>
            }
            .into_any()
        } else {
            view! {
                <p>"Authenticator-app two-factor authentication is "<strong>"not configured"</strong>"."</p>
                <p class="muted">
                    "When enabled, sui-id will ask for a 6-digit code from your authenticator app each time you sign in. \
                     You can use any standards-compliant TOTP app (Aegis, FreeOTP, Google Authenticator, 1Password, etc)."
                </p>
                <form method="post" action="/admin/profile/mfa/enroll/start">
                    <input type="hidden" name="_csrf" value=csrf_for_enroll />
                    <button type="submit">"Set up TOTP"</button>
                </form>
            }
            .into_any()
        };
        let passkey_rows: Vec<_> = passkeys
            .into_iter()
            .map(|p| {
                let id = p.id.clone();
                let delete_url = format!("/admin/profile/webauthn/{id}/delete");
                let csrf = csrf_for_passkey_delete.clone();
                let last = p
                    .last_used_at
                    .map(fmt_time)
                    .unwrap_or_else(|| "never".into());
                view! {
                    <tr>
                        <td>{p.nickname}</td>
                        <td>{fmt_time(p.created_at)}</td>
                        <td class="muted">{last}</td>
                        <td>
                            <form method="post" action=delete_url style="display:inline"
                                  onsubmit="return confirm('Delete this passkey? You will no longer be able to sign in with it.');">
                                <input type="hidden" name="_csrf" value=csrf />
                                <button type="submit" class="danger">"Delete"</button>
                            </form>
                        </td>
                    </tr>
                }
            })
            .collect();
        let passkey_table = if passkey_rows.is_empty() {
            view! {
                <p class="muted">"No passkeys registered yet."</p>
            }
            .into_any()
        } else {
            view! {
                <table>
                    <thead>
                        <tr><th>"Name"</th><th>"Registered"</th><th>"Last used"</th><th></th></tr>
                    </thead>
                    <tbody>{passkey_rows}</tbody>
                </table>
            }
            .into_any()
        };
        view! {
            <Shell title="Profile".to_string() show_nav=true current=Some("profile".to_string())>
                <h2>{format!("Profile - {username}")}</h2>
                {flash_banner(flash)}
                {recovery_block}
                <h3>"Authenticator app (TOTP)"</h3>
                {mfa_section}
                <h3>"Passkeys"</h3>
                <p class="muted">
                    "Passkeys are hardware-backed credentials stored on your phone, laptop, security key, or password manager. \
                     They never leave your device. You can register more than one - bring at least two if you want a fallback in case one is lost."
                </p>
                {passkey_table}
                <h4>"Register a new passkey"</h4>
                <form id="passkey-register-form" method="post" action="/admin/profile/webauthn/register/start">
                    <input type="hidden" name="_csrf" value=csrf_for_passkey_register />
                    <label for="pk-nickname">"Nickname (e.g. \"YubiKey 5C\", \"MacBook Touch ID\")"</label>
                    <input id="pk-nickname" name="nickname" type="text" required=true />
                    <button type="submit">"Register passkey"</button>
                </form>
                <script src="/static/webauthn.js"></script>
            </Shell>
        }
    })
}

pub struct MfaSetupData {
    /// otpauth:// URI for the QR code
    pub otpauth_uri: String,
    /// Pre-rendered SVG of the QR code (full <svg>...</svg> string).
    pub qr_svg: String,
    /// Base32-encoded secret string for users who would rather type it
    /// in than scan the QR code.
    pub secret_b32: String,
}

pub fn render_mfa_setup(data: MfaSetupData, flash: Option<Flash>, csrf_token: String) -> String {
    render(move || {
        let MfaSetupData { otpauth_uri, qr_svg, secret_b32 } = data;
        view! {
            <Shell title="Set up MFA".to_string() show_nav=true current=Some("profile".to_string())>
                <h2>"Set up two-factor authentication"</h2>
                {flash_banner(flash)}
                <ol>
                    <li>"Open your authenticator app and scan the QR code below, or paste the secret key by hand."</li>
                    <li>"Type the 6-digit code your app shows for sui-id into the form to confirm."</li>
                    <li>"You will receive 8 single-use recovery codes. Save them somewhere safe."</li>
                </ol>
                <div inner_html=qr_svg style="max-width:240px"></div>
                <p>"Secret key: "<span class="code">{secret_b32}</span></p>
                <details>
                    <summary class="muted">"otpauth URI (advanced)"</summary>
                    <p><span class="code">{otpauth_uri}</span></p>
                </details>
                <form method="post" action="/admin/profile/mfa/enroll/confirm">
                    <input type="hidden" name="_csrf" value=csrf_token />
                    <label for="code">"Verification code"</label>
                    <input id="code" name="code" type="text" required=true
                           autocomplete="one-time-code" inputmode="text" autofocus=true />
                    <button type="submit">"Confirm and enable"</button>
                </form>
            </Shell>
        }
    })
}

// ---------- dashboard ----------

pub struct DashboardData {
    pub admin_username: String,
    pub user_count: usize,
    pub client_count: usize,
    pub issuer: String,
}

pub fn render_dashboard(data: DashboardData, flash: Option<Flash>) -> String {
    render(move || {
        let DashboardData { admin_username, user_count, client_count, issuer } = data;
        view! {
            <Shell title="Dashboard".to_string() show_nav=true current=Some("dashboard".to_string())>
                <h2>{format!("Hello, {admin_username}.")}</h2>
                {flash_banner(flash)}
                <p class="muted">"sui-id is running. Service overview below."</p>
                <table>
                    <tbody>
                        <tr><th>"Issuer"</th><td><span class="code">{issuer}</span></td></tr>
                        <tr><th>"Users"</th><td>{user_count.to_string()}</td></tr>
                        <tr><th>"Clients"</th><td>{client_count.to_string()}</td></tr>
                        <tr><th>"OIDC Discovery"</th><td><a href="/.well-known/openid-configuration">"/.well-known/openid-configuration"</a></td></tr>
                        <tr><th>"JWKS"</th><td><a href="/.well-known/jwks.json">"/.well-known/jwks.json"</a></td></tr>
                    </tbody>
                </table>
            </Shell>
        }
    })
}

// ---------- users ----------

fn user_row_view(u: UserSummary, current_user: String, csrf: String) -> impl IntoView {
    let status = if u.is_deleted {
        "deleted"
    } else if u.is_disabled {
        "disabled"
    } else if u.is_admin {
        "admin"
    } else {
        "active"
    };
    let display = u.display_name.clone().unwrap_or_default();
    let id_str = u.id.to_string();
    let is_self = u.username == current_user;
    let is_disabled = u.is_disabled;
    let is_deleted = u.is_deleted;
    let mfa_enabled = u.mfa_enabled;
    let action_label = if is_disabled { "Enable" } else { "Disable" };
    let action_target = if is_disabled { "false" } else { "true" };
    let disabled_url = format!("/admin/users/{id_str}/disabled");
    let delete_url = format!("/admin/users/{id_str}/delete");
    let reset_mfa_url = format!("/admin/users/{id_str}/mfa-reset");
    let csrf_disable = csrf.clone();
    let csrf_delete = csrf.clone();
    let csrf_reset = csrf.clone();

    let mfa_cell = if mfa_enabled {
        view! { <td>"on"</td> }.into_any()
    } else {
        view! { <td class="muted">"off"</td> }.into_any()
    };

    let actions = if is_self {
        view! { <td class="muted">"(you)"</td> }.into_any()
    } else if is_deleted {
        view! { <td class="muted">"-"</td> }.into_any()
    } else {
        let reset_form = if mfa_enabled {
            view! {
                <form method="post" action=reset_mfa_url style="display:inline"
                      onsubmit="return confirm('Forcibly remove every MFA factor for this user (TOTP and all passkeys)? Use only when the user has lost access to their second factor.');">
                    <input type="hidden" name="_csrf" value=csrf_reset />
                    <button type="submit" class="secondary">"Reset MFA"</button>
                </form>
                " "
            }
            .into_any()
        } else {
            view! { <></> }.into_any()
        };
        view! {
            <td>
                {reset_form}
                <form method="post" action=disabled_url style="display:inline">
                    <input type="hidden" name="_csrf" value=csrf_disable />
                    <input type="hidden" name="disabled" value=action_target />
                    <button type="submit" class="secondary">{action_label}</button>
                </form>
                " "
                <form method="post" action=delete_url style="display:inline"
                      onsubmit="return confirm('Permanently delete this user?');">
                    <input type="hidden" name="_csrf" value=csrf_delete />
                    <button type="submit" class="danger">"Delete"</button>
                </form>
            </td>
        }
        .into_any()
    };

    view! {
        <tr>
            <td><span class="code">{u.username}</span></td>
            <td>{display}</td>
            <td>{status}</td>
            {mfa_cell}
            <td>{fmt_time(u.created_at)}</td>
            {actions}
        </tr>
    }
}

pub fn render_users(
    users: Vec<UserSummary>,
    flash: Option<Flash>,
    current_user: String,
    csrf_token: String,
) -> String {
    render(move || {
        let csrf_for_rows = csrf_token.clone();
        let csrf_for_form = csrf_token.clone();
        let rows: Vec<_> = users
            .into_iter()
            .map(|u| user_row_view(u, current_user.clone(), csrf_for_rows.clone()))
            .collect();
        view! {
            <Shell title="Users".to_string() show_nav=true current=Some("users".to_string())>
                <h2>"Users"</h2>
                {flash_banner(flash)}
                <h3>"Add a user"</h3>
                <form method="post" action="/admin/users">
                    <input type="hidden" name="_csrf" value=csrf_for_form />
                    <label for="u-name">"Username"</label>
                    <input id="u-name" name="username" type="text" required=true autocomplete="off" />
                    <label for="u-disp">"Display name (optional)"</label>
                    <input id="u-disp" name="display_name" type="text" autocomplete="off" />
                    <label for="u-pw">"Password (12 chars or more)"</label>
                    <input id="u-pw" name="password" type="password" required=true minlength="12" autocomplete="new-password" />
                    <label>
                        <input name="is_admin" type="checkbox" value="true" />
                        " Grant administrator privileges"
                    </label>
                    <button type="submit">"Create user"</button>
                </form>

                <h3>"All users"</h3>
                <table>
                    <thead>
                        <tr><th>"Username"</th><th>"Display"</th><th>"Status"</th><th>"MFA"</th><th>"Created"</th><th></th></tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </Shell>
        }
    })
}

// ---------- clients ----------

fn client_row_view(c: ClientSummary, csrf: String) -> impl IntoView {
    let status = if c.is_deleted {
        "deleted"
    } else if c.is_disabled {
        "disabled"
    } else {
        "active"
    };
    let kind = if c.confidential { "confidential" } else { "public" };
    let id_str = c.id.to_string();
    let is_disabled = c.is_disabled;
    let is_deleted = c.is_deleted;
    let action_label = if is_disabled { "Enable" } else { "Disable" };
    let action_target = if is_disabled { "false" } else { "true" };
    let disabled_url = format!("/admin/clients/{id_str}/disabled");
    let delete_url = format!("/admin/clients/{id_str}/delete");
    let csrf_disable = csrf.clone();
    let csrf_delete = csrf.clone();
    let scopes_display = if c.allowed_scopes.trim().is_empty() {
        "(any)".to_string()
    } else {
        c.allowed_scopes.clone()
    };
    let logout_count = c.post_logout_redirect_uris.len();
    let logout_display = if logout_count == 0 {
        "(falls back to redirect_uris)".to_string()
    } else {
        format!("{logout_count} URI(s)")
    };

    let edit_url = format!("/admin/clients/{id_str}/edit");
    let actions = if is_deleted {
        view! { <td class="muted">"-"</td> }.into_any()
    } else {
        view! {
            <td>
                <a href=edit_url class="button secondary">"Edit"</a>
                " "
                <form method="post" action=disabled_url style="display:inline">
                    <input type="hidden" name="_csrf" value=csrf_disable />
                    <input type="hidden" name="disabled" value=action_target />
                    <button type="submit" class="secondary">{action_label}</button>
                </form>
                " "
                <form method="post" action=delete_url style="display:inline"
                      onsubmit="return confirm('Permanently delete this client and revoke its tokens?');">
                    <input type="hidden" name="_csrf" value=csrf_delete />
                    <button type="submit" class="danger">"Delete"</button>
                </form>
            </td>
        }
        .into_any()
    };

    view! {
        <tr>
            <td>{c.name}</td>
            <td><span class="code">{c.id.to_string()}</span></td>
            <td>{kind}</td>
            <td><span class="code">{scopes_display}</span></td>
            <td class="muted">{logout_display}</td>
            <td>{status}</td>
            {actions}
        </tr>
    }
}

pub fn render_clients(
    clients: Vec<ClientSummary>,
    flash: Option<Flash>,
    new_secret: Option<(String, String)>,
    csrf_token: String,
) -> String {
    render(move || {
        let csrf_for_rows = csrf_token.clone();
        let csrf_for_form = csrf_token.clone();
        let secret_block = new_secret.map(|(cid, sec)| {
            view! {
                <div class="flash warn" role="status">
                    <strong>"Save this client secret now - it will not be shown again."</strong>
                    <div>"Client id: "<span class="code">{cid}</span></div>
                    <div>"Client secret: "<span class="code">{sec}</span></div>
                </div>
            }
        });
        let rows: Vec<_> = clients
            .into_iter()
            .map(|c| client_row_view(c, csrf_for_rows.clone()))
            .collect();
        view! {
            <Shell title="Clients".to_string() show_nav=true current=Some("clients".to_string())>
                <h2>"Clients"</h2>
                {flash_banner(flash)}
                {secret_block}
                <h3>"Register a client"</h3>
                <form method="post" action="/admin/clients">
                    <input type="hidden" name="_csrf" value=csrf_for_form />
                    <label for="c-name">"Application name"</label>
                    <input id="c-name" name="name" type="text" required=true />
                    <label for="c-uris">"Redirect URIs (one per line; https or http loopback)"</label>
                    <textarea id="c-uris" name="redirect_uris" required=true rows="3"></textarea>
                    <label for="c-scopes">"Allowed scopes (space-separated; default: openid profile)"</label>
                    <input id="c-scopes" name="allowed_scopes" type="text" value="openid profile" />
                    <label for="c-logout">"Post-logout redirect URIs (one per line; optional)"</label>
                    <textarea id="c-logout" name="post_logout_redirect_uris" rows="2"></textarea>
                    <label>
                        <input name="confidential" type="checkbox" value="true" checked=true />
                        " Confidential client (will receive a client secret)"
                    </label>
                    <button type="submit">"Register"</button>
                </form>

                <h3>"Registered clients"</h3>
                <table>
                    <thead>
                        <tr>
                            <th>"Name"</th>
                            <th>"Client id"</th>
                            <th>"Type"</th>
                            <th>"Allowed scopes"</th>
                            <th>"Logout URIs"</th>
                            <th>"Status"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </Shell>
        }
    })
}

// ---------- client edit ----------

pub struct ClientEditData {
    pub id: String,
    pub name: String,
    /// Newline-separated for textarea editing.
    pub redirect_uris: Vec<String>,
    /// Space-separated.
    pub allowed_scopes: String,
    pub post_logout_redirect_uris: Vec<String>,
    pub confidential: bool,
    pub is_disabled: bool,
}

pub fn render_client_edit(
    data: ClientEditData,
    flash: Option<Flash>,
    csrf_token: String,
) -> String {
    render(move || {
        let ClientEditData {
            id,
            name,
            redirect_uris,
            allowed_scopes,
            post_logout_redirect_uris,
            confidential,
            is_disabled,
        } = data;
        let post_url = format!("/admin/clients/{id}/edit");
        let kind = if confidential { "confidential" } else { "public" };
        let status = if is_disabled { "disabled" } else { "active" };
        let redirect_uris_value = redirect_uris.join("\n");
        let post_logout_value = post_logout_redirect_uris.join("\n");
        view! {
            <Shell title="Edit client".to_string() show_nav=true current=Some("clients".to_string())>
                <h2>"Edit client"</h2>
                {flash_banner(flash)}
                <p class="muted">
                    "Client id: "<span class="code">{id.clone()}</span>
                    " - Type: "{kind}
                    " - Status: "{status}
                </p>
                <p class="muted">
                    "The client id, type (confidential vs public), and client secret are fixed at creation time. \
                     If you need to change them, delete this client and register a new one."
                </p>
                <form method="post" action=post_url>
                    <input type="hidden" name="_csrf" value=csrf_token />

                    <label for="e-name">"Application name"</label>
                    <input id="e-name" name="name" type="text" required=true value=name />

                    <label for="e-uris">"Redirect URIs (one per line; https or http loopback)"</label>
                    <textarea id="e-uris" name="redirect_uris" required=true rows="3">{redirect_uris_value}</textarea>

                    <label for="e-scopes">"Allowed scopes (space-separated; blank = permit any)"</label>
                    <input id="e-scopes" name="allowed_scopes" type="text" value=allowed_scopes />

                    <label for="e-logout">"Post-logout redirect URIs (one per line; blank = fall back to redirect URIs)"</label>
                    <textarea id="e-logout" name="post_logout_redirect_uris" rows="2">{post_logout_value}</textarea>

                    <button type="submit">"Save changes"</button>
                    " "
                    <a href="/admin/clients" class="secondary">"Cancel"</a>
                </form>
            </Shell>
        }
    })
}

// ---------- audit ----------

fn audit_row_view(e: AuditLogEntryDto) -> impl IntoView {
    view! {
        <tr>
            <td>{fmt_time(e.at)}</td>
            <td><span class="code">{e.actor.map(|a| a.to_string()).unwrap_or_else(|| "-".into())}</span></td>
            <td>{e.action}</td>
            <td><span class="code">{e.target.unwrap_or_default()}</span></td>
            <td>{e.result}</td>
        </tr>
    }
}

pub fn render_audit(entries: Vec<AuditLogEntryDto>, flash: Option<Flash>) -> String {
    render(move || {
        let rows: Vec<_> = entries.into_iter().map(audit_row_view).collect();
        view! {
            <Shell title="Audit".to_string() show_nav=true current=Some("audit".to_string())>
                <h2>"Audit log"</h2>
                {flash_banner(flash)}
                <p class="muted">"Most recent administrative actions, newest first."</p>
                <table>
                    <thead>
                        <tr><th>"When"</th><th>"Actor"</th><th>"Action"</th><th>"Target"</th><th>"Result"</th></tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </Shell>
        }
    })
}

// ---------- signing keys ----------

fn signing_key_row_view(
    k: sui_id_shared::api::SigningKeySummary,
    csrf: String,
) -> impl IntoView {
    let id_str = k.id.to_string();
    let id_for_url = id_str.clone();
    let id_for_display = id_str.clone();
    let status = if k.is_active { "active" } else { "retired" };
    let rotated = k
        .rotated_at
        .map(fmt_time)
        .unwrap_or_else(|| "-".to_string());
    let delete_url = format!("/admin/signing-keys/{id_for_url}/delete");
    let actions = if k.is_active {
        view! { <td class="muted">"(in use)"</td> }.into_any()
    } else {
        view! {
            <td>
                <form method="post" action=delete_url style="display:inline"
                      onsubmit="return confirm('Permanently delete this retired key? Tokens still in flight that were signed with it will fail to verify.');">
                    <input type="hidden" name="_csrf" value=csrf />
                    <button type="submit" class="danger">"Delete"</button>
                </form>
            </td>
        }
        .into_any()
    };
    view! {
        <tr>
            <td><span class="code">{id_for_display}</span></td>
            <td>{k.algorithm}</td>
            <td>{status}</td>
            <td>{fmt_time(k.created_at)}</td>
            <td>{rotated}</td>
            {actions}
        </tr>
    }
}

pub fn render_signing_keys(
    keys: Vec<sui_id_shared::api::SigningKeySummary>,
    flash: Option<Flash>,
    csrf_token: String,
) -> String {
    render(move || {
        let csrf_for_rows = csrf_token.clone();
        let csrf_for_form = csrf_token.clone();
        let rows: Vec<_> = keys
            .into_iter()
            .map(|k| signing_key_row_view(k, csrf_for_rows.clone()))
            .collect();
        view! {
            <Shell
                title="Signing keys".to_string()
                show_nav=true
                current=Some("signing-keys".to_string())
            >
                <h2>"Signing keys"</h2>
                {flash_banner(flash)}
                <p class="muted">
                    "sui-id signs JWTs with one active Ed25519 key. Rotating publishes a fresh key as the new \
                     signing key, demotes the previous one to retired status, and keeps it in JWKS so that \
                     tokens already issued can still be verified during their remaining lifetime. Once those \
                     tokens have expired, you can safely delete the retired key from this page."
                </p>
                <form method="post" action="/admin/signing-keys/rotate">
                    <input type="hidden" name="_csrf" value=csrf_for_form />
                    <button type="submit">"Rotate signing key"</button>
                </form>

                <h3>"All keys"</h3>
                <table>
                    <thead>
                        <tr>
                            <th>"Key id"</th>
                            <th>"Algorithm"</th>
                            <th>"Status"</th>
                            <th>"Created"</th>
                            <th>"Retired"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </Shell>
        }
    })
}

// ---------- error ----------

pub fn render_error(title: String, message: String, request_id: String) -> String {
    render(move || {
        let title2 = title.clone();
        view! {
            <Shell title=title.clone() show_nav=false current=None>
                <h2>{title2}</h2>
                <div class="flash error" role="alert">{message}</div>
                <p class="muted">
                    "If you contact your administrator, please mention this id: "
                    <span class="code">{request_id}</span>
                </p>
                <p><a href="/" class="button secondary">"Back to start"</a></p>
            </Shell>
        }
    })
}

// ---------- /me/security ----------
//
// Self-service security overview for the signed-in user. Shows where
// they are signed in, lets them revoke individual sessions or sign out
// everywhere else, and surfaces a user-scoped activity timeline so
// they have a chance to notice unusual events on their own account
// without an operator having to escalate.
//
// MFA management itself stays on `/admin/profile` (which is
// misleadingly named — it's "user profile", and a non-admin user can
// reach it the same way; the page does not require admin). We link
// to it from here rather than re-implement.

pub struct MeSecurityData {
    pub username: String,
    pub is_admin: bool,
    /// Whether the user has TOTP enrolled.
    pub totp_enabled: bool,
    /// Number of active WebAuthn passkeys.
    pub passkey_count: usize,
    /// Identifier of the session that issued the current request.
    /// Used to mark "this is you" in the session list and to keep it
    /// alive when the user clicks "sign out everywhere else".
    pub current_session_id: String,
    pub sessions: Vec<MeSessionDescriptor>,
    pub recent_events: Vec<MeAuditEntry>,
}

pub struct MeSessionDescriptor {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Comma-separated human display: "password", "password + TOTP", etc.
    pub auth_methods: String,
    pub is_current: bool,
}

pub struct MeAuditEntry {
    pub at: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub result: String,
    pub note: Option<String>,
}

pub fn render_me_security(
    data: MeSecurityData,
    flash: Option<Flash>,
    csrf_token: String,
) -> String {
    render(move || {
        let MeSecurityData {
            username,
            is_admin,
            totp_enabled,
            passkey_count,
            current_session_id,
            sessions,
            recent_events,
        } = data;

        let csrf_for_revoke_others = csrf_token.clone();

        // Session table rows. Each non-current row gets its own
        // mini-form so a user can revoke that specific entry.
        let session_rows: Vec<_> = sessions
            .into_iter()
            .map(|s| {
                let MeSessionDescriptor {
                    id,
                    created_at,
                    expires_at,
                    auth_methods,
                    is_current,
                } = s;
                let when = created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let until = expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let action_cell = if is_current {
                    view! {
                        <td><span class="muted">"current session"</span></td>
                    }
                    .into_any()
                } else {
                    let csrf_for_row = csrf_token.clone();
                    let post_url = format!("/me/security/sessions/{id}/revoke");
                    view! {
                        <td>
                            <form method="post" action=post_url style="display:inline"
                                  onsubmit="return confirm('Sign this session out?');">
                                <input type="hidden" name="_csrf" value=csrf_for_row />
                                <button type="submit" class="secondary">"Revoke"</button>
                            </form>
                        </td>
                    }
                    .into_any()
                };
                view! {
                    <tr>
                        <td>{when}</td>
                        <td>{until}</td>
                        <td>{auth_methods}</td>
                        {action_cell}
                    </tr>
                }
            })
            .collect();

        // Activity timeline.
        let event_rows: Vec<_> = recent_events
            .into_iter()
            .map(|e| {
                let MeAuditEntry {
                    at,
                    action,
                    result,
                    note,
                } = e;
                let when = at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let note_str = note.unwrap_or_default();
                view! {
                    <tr>
                        <td>{when}</td>
                        <td><span class="code">{action}</span></td>
                        <td>{result}</td>
                        <td class="muted">{note_str}</td>
                    </tr>
                }
            })
            .collect();

        let admin_link = is_admin.then(|| {
            view! {
                <p class="muted">
                    <a href="/admin">"Open admin dashboard"</a>
                </p>
            }
        });

        let mfa_summary = if totp_enabled || passkey_count > 0 {
            let parts = {
                let mut v = Vec::<String>::new();
                if totp_enabled {
                    v.push("authenticator app".into());
                }
                if passkey_count > 0 {
                    v.push(format!(
                        "{passkey_count} passkey{}",
                        if passkey_count == 1 { "" } else { "s" }
                    ));
                }
                v.join(", ")
            };
            view! {
                <p>
                    "Two-factor authentication is "<strong>"on"</strong>" — "{parts}"."
                </p>
            }
            .into_any()
        } else {
            view! {
                <p class="flash warn" role="status">
                    "Two-factor authentication is "<strong>"off"</strong>". \
                     A password alone protects this account today. \
                     We recommend enrolling a passkey or an authenticator app."
                </p>
            }
            .into_any()
        };

        view! {
            <Shell title="Security".to_owned() show_nav=false current=None>
                <h2>"Account security"</h2>
                {flash_banner(flash)}
                <p class="muted">
                    "Signed in as "<strong>{username}</strong>"."
                </p>
                {admin_link}

                <section>
                    <h3>"Two-factor authentication"</h3>
                    {mfa_summary}
                    <p>
                        <a href="/admin/profile" class="button secondary">
                            "Manage authenticators"
                        </a>
                    </p>
                </section>

                <section>
                    <h3>"Where you're signed in"</h3>
                    <p class="muted">
                        "Each row is a browser session. \
                         Revoking a session signs that browser out immediately. \
                         The current session is the one you're using now."
                    </p>
                    <table>
                        <thead>
                            <tr>
                                <th>"Started"</th>
                                <th>"Expires"</th>
                                <th>"Factors"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>{session_rows}</tbody>
                    </table>
                    <form method="post" action="/me/security/sessions/revoke-all-others"
                          style="margin-top:1rem"
                          onsubmit="return confirm('Sign out every other browser?');">
                        <input type="hidden" name="_csrf" value=csrf_for_revoke_others />
                        <input type="hidden" name="current_session" value=current_session_id />
                        <button type="submit" class="secondary">
                            "Sign out everywhere else"
                        </button>
                    </form>
                </section>

                <section>
                    <h3>"Recent activity"</h3>
                    <p class="muted">
                        "Authentication and account-management events affecting your account. \
                         If you see something here you didn't do, change your password and \
                         sign out other sessions immediately."
                    </p>
                    <table>
                        <thead>
                            <tr>
                                <th>"When"</th>
                                <th>"Event"</th>
                                <th>"Result"</th>
                                <th>"Note"</th>
                            </tr>
                        </thead>
                        <tbody>{event_rows}</tbody>
                    </table>
                </section>
            </Shell>
        }
    })
}
