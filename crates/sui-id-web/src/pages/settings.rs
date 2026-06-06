//! Settings sub-screens (RFC 065 sub-split).

use leptos::prelude::*;
use super::common::*;

pub enum SettingsTab {
    Basic,
    Security,
    Authentication,
    Logs,
    Email,
    Other,
}

impl SettingsTab {
    fn key(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Security => "security",
            Self::Authentication => "authentication",
            Self::Logs => "logs",
            Self::Email => "email",
            Self::Other => "other",
        }
    }
}


fn settings_tabs(active: SettingsTab, lang: sui_id_i18n::Locale) -> impl IntoView {
    let t = lang.strings();
    let items = [
        (SettingsTab::Basic,          t.settings_tab_basic,           "/admin/settings/basic"),
        (SettingsTab::Security,       t.settings_tab_security,        "/admin/settings/security"),
        (SettingsTab::Authentication, t.settings_tab_authentication,  "/admin/settings/authentication"),
        (SettingsTab::Logs,           t.settings_tab_logs,            "/admin/settings/logs"),
        (SettingsTab::Email,          t.settings_tab_email,           "/admin/settings/email"),
        (SettingsTab::Other,          t.settings_tab_advanced,        "/admin/settings/other"),
    ];
    let active_key = active.key();
    let links: Vec<_> = items
        .into_iter()
        .map(|(tab, label, href)| {
            let aria = if tab.key() == active_key { Some("page") } else { None };
            view! {
                <a class="app-nav__link" href=href aria-current=aria>{label}</a>
            }
        })
        .collect();
    view! {
        <nav class="app-nav" aria-label=t.settings_tabs_aria style="margin-bottom:var(--space-4);flex-wrap:wrap">
            {links}
        </nav>
    }
}

/// Two-column key/value table used inside each settings card. Keeps
/// per-tab content boring and consistent.


mod basic;
mod security;
mod authentication;
mod logs;
mod email;
mod other;

pub use basic::*;
pub use security::*;
pub use authentication::*;
pub use logs::*;
pub use email::*;
pub use other::*;
