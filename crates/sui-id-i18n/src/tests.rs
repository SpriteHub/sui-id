//! Unit tests for `sui-id-i18n`.
//!
//! Lives in its own module file so the rest of the crate stays
//! readable; per the project's conventions, a unit-test block of
//! any meaningful size goes here rather than inline at the end of
//! `lib.rs`. This module is gated behind `#[cfg(test)]` in
//! `lib.rs`, so the file is not compiled in release builds.

use crate::{negotiate_from_accept_language, Locale, STRINGS_JA};

#[test]
fn parse_round_trip() {
    for &loc in Locale::ALL {
        assert_eq!(Locale::parse(loc.tag()), Some(loc));
    }
}

#[test]
fn parse_tolerates_region_suffix() {
    assert_eq!(Locale::parse("en-US"), Some(Locale::En));
    assert_eq!(Locale::parse("ja_JP"), Some(Locale::Ja));
    assert_eq!(Locale::parse("EN"), Some(Locale::En));
}

#[test]
fn parse_unknown_returns_none() {
    assert_eq!(Locale::parse("zh"), None);
    assert_eq!(Locale::parse(""), None);
    assert_eq!(Locale::parse("xyz-RegionTag"), None);
}

#[test]
fn negotiate_picks_first_recognised() {
    // English has q=0.5 in real life, but our parser ignores
    // weights — the first recognised tag wins.
    assert_eq!(
        negotiate_from_accept_language("fr;q=1, en;q=0.5"),
        Some(Locale::En)
    );
    assert_eq!(
        negotiate_from_accept_language("ja, en"),
        Some(Locale::Ja)
    );
    assert_eq!(negotiate_from_accept_language(""), None);
    assert_eq!(negotiate_from_accept_language("zh, fr"), None);
}

#[test]
fn each_locale_has_strings() {
    for &loc in Locale::ALL {
        // Compile-only check that strings() returns; smoke
        // a couple of fields to confirm both populated.
        let s = loc.strings();
        assert!(!s.button_save.is_empty(), "{:?}.button_save empty", loc);
        assert!(!s.login_title.is_empty(), "{:?}.login_title empty", loc);
    }
}

#[test]
fn native_names_are_in_their_own_script() {
    // Sanity: a user wandering in shouldn't see their own
    // language listed only in someone else's script.
    assert!(STRINGS_JA.button_save.chars().any(|c| c >= '\u{3040}'));
    assert!(Locale::Ja.native_name().contains("日本語"));
    assert!(Locale::En.native_name().is_ascii());
}
