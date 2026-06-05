# Roadmap

This file is a loose sketch of direction — nothing here is a promise.
Completed work is tracked in [CHANGELOG.md](CHANGELOG.md) and the
[`rfcs/done/`](rfcs/done/) directory.

---

## Active proposals (proposed RFCs)

| RFC | Title | Priority | Notes |
|---|---|---|---|
| [RFC 004](rfcs/proposed/004-federation.md) | OIDC/SAML federation (upstream IdP) | Low | Identity provider chaining |
| [RFC 005](rfcs/proposed/005-pluggable-user-backends.md) | Pluggable user backends | Low | LDAP/AD directory integration |
| [RFC 006](rfcs/proposed/006-metrics.md) | Metrics and observability | Low | Prometheus / OpenTelemetry |
| [RFC 008](rfcs/proposed/008-third-party-posture.md) | Third-party posture / consent screen | Low-Medium | Explicit consent for external RPs |
| [RFC 009](rfcs/proposed/009-sql-backends.md) | Alternative SQL backends | Low | PostgreSQL / MySQL support |
| [RFC 025](rfcs/proposed/025-multi-tenant-expansion.md) | Multi-tenant expansion | Low | Per-tenant namespaces (post-1.0) |

---

## Near-term (next 5–6 releases)

**The v0.42 → v1.0-rc UI/UX hardening plan** is the main near-term
direction. Six phases (A–F), each shipping in one release. The plan
addresses correctness gaps surfaced during a v0.41.0 implementation
review: the rendered UI was not matching the design contract the v0.40
HANDOFF claimed had been met.

| Phase | Version  | Theme                                              | RFCs (planned)       |
|-------|----------|----------------------------------------------------|----------------------|
| **A** | v0.42.0  | Stop the bleeding (this release)                   | 048, 049, 050        |
| **B** | v0.43.0  | i18n completeness sweep                            | 051, 052, 053, 054   |
| **C** | v0.44.0  | Self-service unification (`/me/security/*`)        | 055, 056, 057        |
| **D** | v0.45.0  | Dangerous operations contract                      | 058, 059, 060        |
| **E** | v0.46.0  | Visual hierarchy + palette extension               | 061, 062, 063, 064   |
| **F** | v0.47.0  | Code structure (split `pages.rs` and admin.rs)     | 065, 066, 067        |
| —     | v0.48.0  | Buffer + RFC index / docs reconciliation           | 068, 069             |

v1.0-rc follows once Phases A–F are clean.

The plan is intentionally correctness-first: visible polish (Phase E)
lands fifth, only after the underlying i18n, navigation, and
dangerous-operation contracts are honest. See
[`docs/src/contributing/`](docs/src/contributing/) and the individual
proposed RFCs once they enter the repository at each phase start.

---

## Completed (recent)

| Version | What shipped |
|---|---|
| v0.42.0 | **Phase A** of the UI/UX hardening plan — RFC 048 (48 `t.xxx` literal-leak fixes), RFC 049 (CSS token freeze + 7 typo fixes), RFC 050 (admin chrome i18n: Nav, Footer, ThemeToggle). Plus the `/me/security/*` locale-resolution fix. Three new CI invariants. |
| v0.41.0 | RFC 040 completion (`/me/security/mfa`+`/sessions`), RFC 045 (user disable reason), RFC 046 (audit copy-ID), RFC 047 (dev summary + secret rotation) |
| v0.40.0 | RFC 040 (`/me/security` tabs initial), RFC 041 (HIBP consistency), RFC 042 (error i18n), RFC 043 (dashboard events), RFC 044 (state-word contract) |
| v0.39.0 | RFC 038 (consent screen), RFC 039 (settings i18n complete) |
| v0.38.0 | e2e coverage (RFC 030/033/035), audit-events doc, settings i18n section headers |
| v0.37.0 | RFC 029 pass 2 (dynamic locale), RFC 035 (user detail), RFC 036 (docs/Phase 5) |
| v0.36.0 | RFC 030 (dangerous ops confirm), RFC 031 (dashboard prompts), RFC 033 (audit), RFC 034 (passkey+empty) |
| v0.35.0 | RFC 032 (dev mode banner), RFC 029 first pass (admin i18n) |
| v0.34.0 | RFC 002 (i18n: zh locale, Formatters, audit labels, dir=, per-recipient locale) |
| v0.33.0 | RFC 001 (email outbox + retry worker) |
| v0.32.0 | RFC 017 (UI/UX contracts), RFC 023 (visual design system), RFC 024 (doc consolidation) |
| v0.31.0 | RFC 014 (hot-path caches), RFC 028 (copy buttons) |
| v0.30.0 | RFC 013 (async DB layer — full implementation + test fixes) |
| v0.29.13 | RFC 026 (admin logout), RFC 027 (client scope UX), dup-username bug fix |
| v0.29.12 | RFC 013 async DB layer initial |
| v0.29.10–11 | RFC 021/022 (schema invariants, boolean CHECKs, migration safety) |

Full history: [CHANGELOG.md](CHANGELOG.md)

---

## Status

v0.42.0 ships Phase A of the v0.42 → v1.0-rc UI/UX hardening plan.
A v0.41.0 implementation review found that the rendered admin panel
did not match the design contract the HANDOFF claimed had been
delivered: 48 `t.xxx` literal leaks on page titles and buttons, 7
undefined CSS variables breaking visual styling, an entirely non-i18n
admin navigation chrome, and a `/me/security/*` locale-resolution
helper that ignored Accept-Language and the user-preference cookie.
Phase A addresses these correctness gaps so the rest of the hardening
plan rests on screens that at least render their own headings.

The project is **on hold for v1.0** until Phases B–F land. Phase B
(per-screen i18n completeness sweep) is the next milestone.

---

## Constraints and non-goals (pre-1.0)

- **Single realm.** All users share one namespace. Per-tenant isolation is
  RFC 025, post-1.0. See [docs/operators.md](docs/operators.md) §
  "User–client relationship".
- **SQLite only.** Alternative backends are RFC 009, low priority. The
  current SQLite implementation is production-grade for small deployments.
- **No user-facing theming API.** CSS tokens are for the maintainer, not
  operators.
- **No plugin system.** RFC 005 sketches one; it is not scheduled.
