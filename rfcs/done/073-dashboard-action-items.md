# RFC 073 — Dashboard action items

**Status.** Implemented (v0.58.0)
**Priority.** P2 — visible improvement to the admin landing page; no
schema work; no security implications. Ships independently of RFC 071
and RFC 072.
**Tracks.** UX rethink — dashboard restructuring (see audit notes,
v0.57.1 session).
**Touches.** `crates/sui-id/src/handlers/admin/dashboard.rs`,
`crates/sui-id-web/src/pages/dashboard.rs`,
`crates/sui-id-store/src/repos/{users,email_outbox,password_reset_tokens}.rs`,
`crates/sui-id-web/src/components/banners.rs`, `crates/sui-id-i18n`. No schema changes.

---

## Background

Today the admin dashboard at `/admin` shows a small set of vanity
metrics (counts of users, clients, recent events) and a sparkline. The
information is correct but not actionable — an operator opening the
dashboard learns *what the system is*, not *what needs attention*. For
small-to-medium deployments this is the most-loaded page in the admin
UI, and it is currently the least useful.

## Non-goals

- **No third-party data.** All checks operate on local DB and runtime state.
- **No new notification channel.** Findings are displayed inline; there
  is no email alert.
- **No persisted dismissals beyond Getting Started.** Action items are
  computed fresh on every request; they appear when the condition is true,
  disappear when false. No "snooze for 30 days" UX.
- **No anomaly detection.** Surfacing "5 failed logins from same IP" is
  RFC 074 (Audit anomalies) work; this RFC only handles deterministic
  computed conditions.

## Goal

Add an "Action items" section to the top of the admin dashboard that
surfaces operational concerns the admin should know about. Items are
computed on every request from existing data; they appear only when their
condition is true. Each item links to the page where the issue can be
addressed.

A separate "Getting Started" checklist appears for fresh instances and
disappears once all items are checked.

## Design

### Computed action items

Each item is a struct of `(severity, label, link, condition)`. The
dashboard renders them sorted by severity (danger > warning > info),
then condition truth order.

| Item | Severity | Condition | Link |
|---|---|---|---|
| Users without MFA | warning | `count(users where MFA not enrolled and is_admin) ≥ 1` (admin users only — encourage MFA on privileged accounts first) | `/admin/users?filter=no-mfa` |
| Old signing key | warning | `oldest active signing key created_at < now() − 11 months` (recommend rotation before 12-month sunset) | `/admin/audit#signing-keys` |
| Email outbox stuck | danger | `count(email_outbox where status='pending' and created_at < now() − 1 hour) ≥ 1` | `/admin/settings/email#outbox` |
| Stale backup | warning | `last_successful_backup_at < now() − 7 days` | `/admin/settings/advanced#backup` |
| HIBP failing | warning | last 5 password-set HIBP checks returned `Unavailable` | `/admin/settings/authentication#hibp` |
| Pending password resets | info | `count(password_reset_tokens where used = 0 and expires_at > now()) ≥ 5` | `/admin/users?filter=pending-reset` |

All conditions are deterministic SQL aggregates against existing tables;
no new instrumentation is required. The `?filter=` query parameters on
the user list link to filters that should already exist or be added in
the same RFC (small scope addition; see migration plan).

### Getting Started checklist

For fresh instances (defined below), the dashboard renders a checklist
above the action items:

| Step | Done when |
|---|---|
| Configure SMTP | `email_outbox` table has any row OR a smoke-test email has been sent from settings |
| Add your first app | `count(clients where is_deleted = 0) ≥ 1` |
| Enable MFA on your admin account | the signed-in admin has `totp_enrolled = 1` OR `webauthn_credential_count > 0` |
| Configure backup destination | `backup_config` table has a row with a non-empty destination |
| Review default password policy | `server_settings.password_policy_reviewed_at` is NOT NULL (new boolean / timestamp column, set when the admin opens the Authentication settings page) |

A "fresh instance" is defined as one where at least one of the above
items is NOT done. Once all are done, the checklist disappears (and does
not return).

**Optional dismissal**: an admin can dismiss the entire checklist by
clicking a small "Dismiss this guide" link; this sets a server-side
flag (`server_settings.getting_started_dismissed_at`) that hides it
forever. No per-item dismissals — the whole checklist is one unit.

### Page structure

```
<header class="page-header">
  <h1>Dashboard</h1>
</header>

<!-- New section, only present if conditions exist or checklist active -->
<section class="action-items">
  {if getting_started_visible}
    <div class="card callout--info">
      <h2>Getting Started</h2>
      <ul class="checklist">
        ... items ...
      </ul>
      <a href="?dismiss_getting_started=1" class="muted text-caption">
        Dismiss this guide
      </a>
    </div>
  {/if}

  {if any_action_item}
    <h2>Action items</h2>
    <ul class="action-items-list">
      ... items, sorted by severity ...
    </ul>
  {/if}
</section>

<!-- Existing dashboard content below -->
<section class="dashboard-metrics">
  ... existing stat cards, sparkline ...
</section>
```

### CSS

Three new classes in `components/banners.rs` (reusing the
`.callout` family from v0.50.0):

```css
.action-items-list { list-style: none; padding: 0; }
.action-items-list > li {
  display: flex;
  gap: var(--space-3);
  padding: var(--space-2) 0;
  border-bottom: var(--border-width-default) solid var(--border-muted);
}
.action-items-list > li:last-child { border-bottom: 0; }
.action-item__severity {
  flex: 0 0 auto;
  /* dot/badge using existing --danger / --warning / --info tokens */
}
.checklist { list-style: none; padding: 0; }
.checklist li { display: flex; gap: var(--space-2); padding: var(--space-1) 0; }
.checklist li::before {
  /* "☐" for incomplete, "✓" for complete */
  font-family: monospace;
  color: var(--fg-muted);
}
.checklist li.done::before { content: "✓ "; color: var(--success-default); }
.checklist li:not(.done)::before { content: "☐ "; }
```

ABDD: the check-marks are characters with text labels, not colour-only.

### New i18n keys

- `dashboard_action_items_title` — "Action items"
- `dashboard_getting_started_title` — "Getting Started"
- `dashboard_getting_started_dismiss` — "Dismiss this guide"
- One key per action item label and per checklist step label (×3 locales).

### Auditor visibility (interacts with RFC 071)

If RFC 071 lands first, auditors see the action items (they are
diagnostic) but **not** the Getting Started checklist (it is admin-task
oriented and includes mutation paths). Auditors clicking through an
action item land on a read-only view of the relevant page.

If RFC 073 lands first, all of the above logic is gated on `is_admin`,
which is the only role today. The RFC 071 patch is one line.

## Acceptance criteria

- [ ] Dashboard renders zero items when the system is healthy (no false
  positives in a fresh, fully-configured instance).
- [ ] Each action item appears when its condition is true and disappears
  when false (verified by unit tests on the condition functions).
- [ ] Getting Started checklist renders on first dashboard load after
  setup; disappears when all five items are done; can be manually
  dismissed.
- [ ] Each item links to the page that resolves it.
- [ ] No new external queries (only local DB and runtime state).
- [ ] CI invariants unchanged.

## Risks

| Risk | Mitigation |
|---|---|
| Dashboard load slows down due to multiple aggregate queries | Each query is on indexed columns; in the worst case, total added latency is < 20ms on a 1000-user DB. If observed slower, cache for 60s using the existing dashboard cache layer (`AppState.caches`). |
| Action items become noisy and admins ignore them | The set is small (6 items) and each is genuinely actionable. If more are proposed, they go through a separate RFC. |
| Getting Started false-positive (admin completes all setup but checklist still shows) | The "done" predicates are precise SQL; if any condition is wrong, fix and ship as a patch — no schema rollback needed. |

## Follow-up RFCs

- **RFC 074 (post-1.0)**: Audit anomaly detection — surface unusual
  events (failed-login spikes, new-country logins). Cross-references
  this dashboard.
- **RFC 075 (post-1.0)**: Per-page health indicators — small badge
  per top-nav item showing "X items needing attention." Optional.

## Implementation note (v0.58.0)

### What shipped

**Four new repo helpers** — all read-only aggregates on indexed columns:
- `users::count_admins_without_mfa()` — admins with no TOTP and no WebAuthn credential.
- `users::has_mfa(user_id)` — single-user MFA check for the Getting-Started checklist.
- `email_outbox::count_stuck_pending(threshold, now)` — emails in `queued` state older than `threshold`.
- `password_reset_tokens::count_outstanding(now)` — unconsumed, unexpired reset tokens.

**`DashboardData` extended** with seven new fields plus `getting_started_visible()` and `has_action_items()` helper methods. Handler computes all values best-effort (failures default to zero/false).

**Dashboard render** gains two sections above the sparkline:
- *Getting Started* — `.callout--info` with three checklist items (☐/✓ text indicators, ABDD-compliant).
- *Action items* — `.callout--warning` unifying the three RFC 031 warnings with four new conditions.

**Eight new i18n keys** (en/ja/zh); four are `fn` types for parameterised messages.

**Two new CSS classes** — `.action-items-list` and `.checklist` — in `components/banners.rs`.

### Deferred
- Stale-backup warning: no `backup_history` table. Deferred.
- HIBP failure rate: requires runtime instrumentation. Deferred.
- Checklist dismissal: checklist disappears naturally when all done; explicit dismiss deferred.

### CI outcome
- [x] `text-leaks` = 0, `inline-style-bound` = 0, `css-tokens` = 148, `semantic-parity` = 36.
- [x] **228/228 library tests pass.**
