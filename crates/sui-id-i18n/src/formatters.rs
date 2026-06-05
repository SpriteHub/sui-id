//! Locale-aware date, time, and count formatters (RFC 002 § B).
//!
//! Each locale provides a static `Formatters` constant. The caller
//! retrieves it via [`crate::Locale::formatters`] and calls the
//! appropriate function.
//!
//! ## Design
//!
//! Functions are plain `fn` pointers rather than closures so the
//! struct can be `&'static`. No ICU dependency — all patterns are
//! hand-written to keep the binary lean and the logic auditable.
//!
//! ## Timestamp rendering policy (from RFC 017 § 4)
//!
//! - **Admin UI** (audit log, session list): absolute timestamps only.
//!   Operators need exact times; relative timestamps are ambiguous
//!   across time zones.
//! - **End-user UI** (`/me/security` "last used"): relative timestamps
//!   are acceptable and preferred for readability.
//!
//! Both rendering modes are available here; the view layer chooses.

use chrono::{DateTime, Datelike, Timelike, Utc};

/// Locale-aware formatting functions for dates, times, and counts.
///
/// Obtain via [`crate::Locale::formatters`].
pub struct Formatters {
    /// Date only: e.g. "2024年5月12日" or "12 May 2024".
    pub fmt_date: fn(DateTime<Utc>) -> String,
    /// Time only (24 h): e.g. "14:07".
    pub fmt_time: fn(DateTime<Utc>) -> String,
    /// Date + time: e.g. "2024年5月12日 14:07" or "12 May 2024 14:07".
    pub fmt_date_time: fn(DateTime<Utc>) -> String,
    /// Relative time from `now`: e.g. "3 時間前" or "3 hours ago".
    /// `now` is passed in so callers can use a mock clock in tests.
    pub fmt_relative: fn(at: DateTime<Utc>, now: DateTime<Utc>) -> String,
    /// Locale-appropriate number with thousands separator: e.g. "1,234".
    pub fmt_count: fn(u64) -> String,
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn fmt_time_shared(dt: DateTime<Utc>) -> String {
    format!("{:02}:{:02}", dt.hour(), dt.minute())
}

fn fmt_count_shared(n: u64) -> String {
    // Group digits with commas from the right: 1,234,567.
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        let remaining = s.len() - i;
        if i > 0 && remaining % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

// ── Japanese (ja) ────────────────────────────────────────────────────────────

const JA_MONTHS: &[&str] = &[
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
];

fn ja_fmt_date(dt: DateTime<Utc>) -> String {
    format!(
        "{}年{}月{}日",
        dt.year(),
        JA_MONTHS[(dt.month() - 1) as usize],
        dt.day()
    )
}

fn ja_fmt_date_time(dt: DateTime<Utc>) -> String {
    format!("{} {}", ja_fmt_date(dt), fmt_time_shared(dt))
}

fn ja_fmt_relative(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - at).num_seconds();
    if secs < 0 {
        return "たった今".into();
    }
    if secs < 60 {
        return format!("{secs} 秒前");
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins} 分前");
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours} 時間前");
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{days} 日前");
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months} ヶ月前");
    }
    let years = months / 12;
    format!("{years} 年前")
}

/// Japanese (ja) formatters.
pub static FORMATTERS_JA: Formatters = Formatters {
    fmt_date:      ja_fmt_date,
    fmt_time:      fmt_time_shared,
    fmt_date_time: ja_fmt_date_time,
    fmt_relative:  ja_fmt_relative,
    fmt_count:     fmt_count_shared,
};

// ── English (en) ─────────────────────────────────────────────────────────────

const EN_MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn en_fmt_date(dt: DateTime<Utc>) -> String {
    format!(
        "{} {} {}",
        dt.day(),
        EN_MONTHS[(dt.month() - 1) as usize],
        dt.year()
    )
}

fn en_fmt_date_time(dt: DateTime<Utc>) -> String {
    format!("{} {}", en_fmt_date(dt), fmt_time_shared(dt))
}

fn en_fmt_relative(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - at).num_seconds();
    if secs < 0 {
        return "just now".into();
    }
    if secs < 60 {
        let s = if secs == 1 { "second" } else { "seconds" };
        return format!("{secs} {s} ago");
    }
    let mins = secs / 60;
    if mins < 60 {
        let s = if mins == 1 { "minute" } else { "minutes" };
        return format!("{mins} {s} ago");
    }
    let hours = mins / 60;
    if hours < 24 {
        let s = if hours == 1 { "hour" } else { "hours" };
        return format!("{hours} {s} ago");
    }
    let days = hours / 24;
    if days < 30 {
        let s = if days == 1 { "day" } else { "days" };
        return format!("{days} {s} ago");
    }
    let months = days / 30;
    if months < 12 {
        let s = if months == 1 { "month" } else { "months" };
        return format!("{months} {s} ago");
    }
    let years = months / 12;
    let s = if years == 1 { "year" } else { "years" };
    format!("{years} {s} ago")
}

/// English (en) formatters.
pub static FORMATTERS_EN: Formatters = Formatters {
    fmt_date:      en_fmt_date,
    fmt_time:      fmt_time_shared,
    fmt_date_time: en_fmt_date_time,
    fmt_relative:  en_fmt_relative,
    fmt_count:     fmt_count_shared,
};

// ── Chinese Simplified (zh) ───────────────────────────────────────────────────

fn zh_fmt_date(dt: DateTime<Utc>) -> String {
    format!("{}年{}月{}日", dt.year(), dt.month(), dt.day())
}

fn zh_fmt_date_time(dt: DateTime<Utc>) -> String {
    format!("{} {}", zh_fmt_date(dt), fmt_time_shared(dt))
}

fn zh_fmt_relative(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - at).num_seconds();
    if secs < 0 {
        return "刚刚".into();
    }
    if secs < 60 {
        return format!("{secs} 秒前");
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins} 分钟前");
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours} 小时前");
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{days} 天前");
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months} 个月前");
    }
    let years = months / 12;
    format!("{years} 年前")
}

/// Chinese Simplified (zh) formatters.
pub static FORMATTERS_ZH: Formatters = Formatters {
    fmt_date:      zh_fmt_date,
    fmt_time:      fmt_time_shared,
    fmt_date_time: zh_fmt_date_time,
    fmt_relative:  zh_fmt_relative,
    fmt_count:     fmt_count_shared,
};

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    #[test]
    fn ja_date_formatting() {
        let dt = ts(2024, 5, 12, 14, 7);
        assert_eq!(ja_fmt_date(dt), "2024年5月12日");
        assert_eq!(ja_fmt_date_time(dt), "2024年5月12日 14:07");
        assert_eq!(fmt_time_shared(dt), "14:07");
    }

    #[test]
    fn en_date_formatting() {
        let dt = ts(2024, 5, 12, 14, 7);
        assert_eq!(en_fmt_date(dt), "12 May 2024");
        assert_eq!(en_fmt_date_time(dt), "12 May 2024 14:07");
    }

    #[test]
    fn zh_date_formatting() {
        let dt = ts(2024, 5, 12, 14, 7);
        assert_eq!(zh_fmt_date(dt), "2024年5月12日");
        assert_eq!(zh_fmt_date_time(dt), "2024年5月12日 14:07");
    }

    #[test]
    fn relative_ja() {
        let now = ts(2024, 5, 12, 15, 0);
        assert_eq!(ja_fmt_relative(ts(2024, 5, 12, 14, 57), now), "3 分前");
        assert_eq!(ja_fmt_relative(ts(2024, 5, 12, 12, 0), now), "3 時間前");
        assert_eq!(ja_fmt_relative(ts(2024, 5,  9, 15, 0), now), "3 日前");
    }

    #[test]
    fn relative_en() {
        let now = ts(2024, 5, 12, 15, 0);
        assert_eq!(en_fmt_relative(ts(2024, 5, 12, 14, 57), now), "3 minutes ago");
        assert_eq!(en_fmt_relative(ts(2024, 5, 12, 12, 0), now), "3 hours ago");
        assert_eq!(en_fmt_relative(ts(2024, 5,  9, 15, 0), now), "3 days ago");
        assert_eq!(en_fmt_relative(ts(2024, 5, 12, 14, 59), now), "1 minute ago");
    }

    #[test]
    fn fmt_count_thousands() {
        assert_eq!(fmt_count_shared(0), "0");
        assert_eq!(fmt_count_shared(999), "999");
        assert_eq!(fmt_count_shared(1000), "1,000");
        assert_eq!(fmt_count_shared(1_234_567), "1,234,567");
    }
}
