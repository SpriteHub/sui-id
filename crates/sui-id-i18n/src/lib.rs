//! sui-id internationalisation.
//!
//! ## Design
//!
//! All user-facing strings are fields on a [`Strings`] struct. Each
//! supported locale has a `static Strings` constant with all fields
//! filled in ([`STRINGS_JA`], [`STRINGS_EN`]). Adding a locale means
//! adding a variant to [`Locale`] and a new `static Strings`
//! constant — the compiler then guarantees every translation is
//! complete via the exhaustive `match` in [`Locale::strings`].
//! Adding a string means adding a field to [`Strings`] — the
//! compiler then yells at every per-locale constant until it's
//! filled in.
//!
//! Strings without variable interpolation are `&'static str`.
//! Strings with interpolation use small format functions that
//! take parameters and return `String`. We deliberately avoid a
//! generic templating layer (Fluent, MessageFormat, etc) at this
//! tier — the interpolation patterns we have are simple,
//! enumeration-style ("3 outstanding tokens"), and a per-locale
//! function is more readable than a templated string.
//!
//! ## What lives here, what doesn't
//!
//! - **Lives here**: UI labels, button text, flash messages,
//!   page titles, email subjects/bodies. Anything a human reads.
//! - **Does not live here**: log messages, audit-event names
//!   (those are stable identifiers operators query against),
//!   error machine codes, configuration keys.
//!
//! ## Module layout
//!
//! - [`strings`] — the [`Strings`] struct (every translatable
//!   field).
//! - [`ja`], [`en`] — per-locale `static Strings` constants.
//! - [`tests`] — unit tests, kept out of `lib.rs` to keep the
//!   public surface tidy.
//!
//! ## Future expansion (see sui-id ROADMAP)
//!
//! - More locales (zh, ko, etc) — add `Locale::Zh` and
//!   `STRINGS_ZH`; the type system handles the rest.
//! - Date/number formatting localisation — currently we use a
//!   single ISO-ish format across locales for simplicity. v2
//!   will add per-locale formatters.

mod en;
mod ja;
mod strings;
#[cfg(test)]
mod tests;

pub use crate::en::STRINGS_EN;
pub use crate::ja::STRINGS_JA;
pub use crate::strings::Strings;

use serde::{Deserialize, Serialize};

/// A supported locale.
///
/// New variants must:
///   - have a stable BCP-47-style tag returned by [`Locale::tag`];
///   - have a `static STRINGS_*` constant matched in [`Locale::strings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    Ja,
    En,
}

impl Locale {
    /// All locales sui-id recognises, in display order.
    pub const ALL: &'static [Locale] = &[Locale::Ja, Locale::En];

    /// BCP-47 language tag. Used in HTML `lang=` attributes,
    /// cookies, and the user preference column. Stable.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Ja => "ja",
            Self::En => "en",
        }
    }

    /// Native-language name of this locale, displayed in the
    /// language picker. Always shown in the locale's own script
    /// so a user who has accidentally landed on the wrong language
    /// can still recognise their own.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::Ja => "日本語",
            Self::En => "English",
        }
    }

    /// Parse a tag back into a `Locale`. Tolerant of region
    /// suffixes (`en-US` → `En`) and capitalisation. Unknown tags
    /// return `None`; callers should fall back through their
    /// preference chain rather than choosing here.
    pub fn parse(tag: &str) -> Option<Locale> {
        let primary = tag
            .split(|c: char| c == '-' || c == '_')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "ja" => Some(Locale::Ja),
            "en" => Some(Locale::En),
            _ => None,
        }
    }

    /// Strings table for this locale. The exhaustive match is the
    /// completeness guarantee — adding a `Locale` variant without a
    /// strings table fails to compile.
    pub fn strings(self) -> &'static Strings {
        match self {
            Self::Ja => &STRINGS_JA,
            Self::En => &STRINGS_EN,
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::Ja
    }
}

/// Pick a locale from a `q`-weighted Accept-Language header.
///
/// Cheap parser: split on commas, take each token's primary
/// subtag, return the first one we recognise. We ignore `q=`
/// weights — for a two-locale catalogue the cost of a real parser
/// outweighs the benefit. A user with `Accept-Language: fr;q=1, en;q=0.5`
/// will get English (the first recognised tag), which matches the
/// "best available match" intent close enough.
pub fn negotiate_from_accept_language(header: &str) -> Option<Locale> {
    for raw in header.split(',') {
        let tag = raw.split(';').next().unwrap_or("").trim();
        if let Some(loc) = Locale::parse(tag) {
            return Some(loc);
        }
    }
    None
}
