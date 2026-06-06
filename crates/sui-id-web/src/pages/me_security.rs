//! Self-service security tabs (RFC 065 sub-split).

use leptos::prelude::*;
use super::common::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeTab {
    Overview,
    Mfa,
    Passkey,
    Sessions,
    Language,
}


pub struct MeShellData {
    pub username: String,
    pub is_admin: bool,
    pub active_tab: MeTab,
}


fn me_security_tabs(active: MeTab, lang: sui_id_i18n::Locale) -> impl IntoView {
    let t = lang.strings();
    let items = [
        (MeTab::Overview, t.me_tab_overview, "/me/security/overview"),
        (MeTab::Mfa,      t.me_tab_mfa,      "/me/security/mfa"),
        (MeTab::Passkey,  t.me_tab_passkey,  "/me/security/passkeys"),
        (MeTab::Sessions, t.me_tab_sessions, "/me/security/sessions"),
        (MeTab::Language, t.me_tab_language, "/me/security/language"),
    ];
    let tab_items: Vec<_> = items.iter().map(|(tab, label, href)| {
        let cls = if *tab == active { "tab tab--active" } else { "tab" };
        view! { <a href=*href class=cls>{*label}</a> }
    }).collect();
    view! {
        <nav class="tabs" aria-label=t.me_security_tabs_aria>
            {tab_items}
        </nav>
    }
}




mod overview;
mod mfa;
mod sessions;
mod passkey;
mod language;
mod security;

pub use overview::*;
pub use mfa::*;
pub use sessions::*;
pub use passkey::*;
pub use language::*;
pub use security::*;
