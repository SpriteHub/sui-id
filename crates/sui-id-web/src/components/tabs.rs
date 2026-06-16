//! Tab strips.
//!
//! Owns: the `.tabs` and `.tabs__link` styles introduced in RFC 023
//! and the `.route-tabs` / `.route-tabs__link` styles introduced by
//! RFC-MI-022 (v0.51.1).
//!
//! ## CSS families
//!
//! `.tab-btn` / `.tabs__bar` — original JS-driven tab component (RFC
//! 023). Used when the tab content lives on the same page and is
//! toggled client-side.
//!
//! `.route-tabs` / `.route-tabs__link` — route-based tab strip (RFC-MI-022).
//! Each tab is a normal `<a>` anchor pointing at a distinct route. The
//! active link carries `aria-current="page"`. No JS required. This is
//! the pattern mandated by the migration plan §D-02.
//!
//! ## Rust helper
//!
//! [`route_tabs`] renders a `.route-tabs` nav using a static slice of
//! [`RouteTab`] entries. Import via `crate::components::tabs::{RouteTab, route_tabs}`.

use leptos::prelude::*;

/// A single entry in a route-based tab strip.
#[derive(Clone, Copy)]
pub struct RouteTab {
    /// Short slug used to identify the active tab (matches the route's
    /// path segment, e.g. `"mfa"`, `"basic"`).
    pub key: &'static str,
    /// Full href of the tab's page (e.g. `"/me/security/mfa"`).
    pub href: &'static str,
    /// Localised display label.
    pub label: &'static str,
}

/// Render a route-based tab strip.
///
/// - `aria_label` — value for `<nav aria-label="…">`.
/// - `current` — the slug of the active tab; must equal one of the
///   `RouteTab::key` values.  If no tab matches, no tab carries
///   `aria-current`, which is acceptable (e.g. on a sub-route).
/// - `tabs` — ordered list of tabs; rendered left-to-right.
///
/// The output is a `<nav class="route-tabs">` containing `<a>`
/// elements.  Active state is communicated via `aria-current="page"`,
/// a visible underline, and a colour change — no colour-only indicator.
pub fn route_tabs(
    aria_label: &'static str,
    current: &str,
    tabs: &'static [RouteTab],
) -> impl IntoView {
    let current = current.to_owned(); // needed to move into the closure
    let links: Vec<_> = tabs
        .iter()
        .map(|tab| {
            let aria = if tab.key == current.as_str() {
                Some("page")
            } else {
                None
            };
            let href = tab.href;
            let label = tab.label;
            view! {
                <a class="route-tabs__link" href=href aria-current=aria>{label}</a>
            }
        })
        .collect();
    view! {
        <nav class="route-tabs" aria-label=aria_label>
            {links}
        </nav>
    }
}

pub const TABS_CSS: &str = r#"
/* ── Tabs (RFC 023) ─────────────────────────────────────────────────── */
/* Horizontal tab bar for Settings and other multi-panel screens.         */
.tabs {
  display: flex;
  flex-direction: column;
}
.tabs__bar {
  display: flex;
  gap: 0;
  border-bottom: var(--border-width-default) solid var(--border-default);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
.tab-btn {
  padding: var(--space-2) var(--space-3);
  background: transparent;
  border: none;
  border-bottom: var(--border-width-emphasis) solid transparent;
  color: var(--fg-muted);
  font: var(--font-weight-regular) var(--font-size-body) / 1 var(--font-sans);
  cursor: pointer;
  white-space: nowrap;
  transition: color var(--motion-fast) var(--motion-easing),
              border-color var(--motion-fast) var(--motion-easing);
  margin-bottom: calc(-1 * var(--border-width-default)); /* align with bar border */
}
.tab-btn:hover  { color: var(--fg-default); }
.tab-btn:focus-visible {
  outline: var(--border-width-emphasis) solid var(--state-focus);
  outline-offset: -2px;
}
.tab-btn[aria-selected="true"] {
  color: var(--accent-default);
  border-bottom-color: var(--accent-default);
  font-weight: var(--font-weight-medium);
}
.tabs__panel {
  padding-top: var(--space-4);
}

/* ── Route-based tab strip (RFC-MI-022, v0.51.1) ────────────────────── */
/* Replaces the per-page ad-hoc tab helpers for /me/security/* and       */
/* /admin/settings/*. Uses <a href="…"> anchors with                     */
/* aria-current="page" on the active link.                               */
/* No JavaScript required; deep-linkable; back/forward button works.     */
.route-tabs {
  display: flex;
  gap: 0;
  border-bottom: var(--border-width-default) solid var(--border-default);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  margin-bottom: var(--space-4);
  flex-wrap: wrap;
}
.route-tabs__link {
  display: inline-block;
  padding: var(--space-2) var(--space-3);
  color: var(--fg-muted);
  text-decoration: none;
  white-space: nowrap;
  border-bottom: var(--border-width-emphasis) solid transparent;
  /* Align the bottom border with the container's bottom border. */
  margin-bottom: calc(-1 * var(--border-width-default));
  transition: color var(--motion-fast) var(--motion-easing),
              border-color var(--motion-fast) var(--motion-easing);
}
.route-tabs__link:hover {
  color: var(--fg-default);
  text-decoration: none;
}
.route-tabs__link:focus-visible {
  outline: var(--border-width-emphasis) solid var(--state-focus);
  outline-offset: -2px;
}
/* Active tab: colour + bottom indicator. Non-colour indicator ensures
 * the state is perceptible without relying on colour alone (ABDD). */
.route-tabs__link[aria-current="page"] {
  color: var(--accent-emphasis);
  border-bottom-color: var(--accent-default);
  font-weight: var(--font-weight-medium);
}

"#;
