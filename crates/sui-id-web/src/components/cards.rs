//! Card and panel surfaces.
//!
//! Owns: `.card` base, the v0.46 `.card--{warn,info,success,callout}`
//! variants (RFC 062), and the `.empty-state` primitive (RFC 064).
//! Subsequent MI work that adds mockup-style metric cards or
//! callouts lands here.

pub const CARDS_CSS: &str = r#"
/* ------------------------------------------------------------------ */
/* Cards / panels                                                      */
/* ------------------------------------------------------------------ */

.card {
  background: var(--surface-elevated);
  border: var(--border-width-default) solid var(--border-muted);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  box-shadow: var(--shadow-sm);
}
.card + .card { margin-top: var(--space-3); }
.card__title {
  margin: 0 0 var(--space-2) 0;
  font-size: var(--font-size-h3);
  line-height: var(--line-height-h3);
}
.card__body { color: var(--fg-default); }
.card__footer {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: var(--border-width-default) solid var(--border-muted);
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

/* RFC 062 (v0.46.0) — card variants.
 * Compose with .card: <section class="card card--warn">. Each variant
 * gives the card an asymmetric 4px left accent and a subtle tinted
 * background, so a row of cards can read at a glance as "this one is
 * different." Colours come from RFC 061 semantic tokens, so light/dark
 * pairing is automatic. */
.card--warn {
  background: var(--warning-subtle);
  border-color: var(--warning-default);
  border-left-width: 4px;
}
.card--info {
  background: var(--info-subtle);
  border-color: var(--info-default);
  border-left-width: 4px;
}
.card--success {
  background: var(--success-subtle);
  border-color: var(--success-default);
  border-left-width: 4px;
}
.card--callout {
  /* Accent (lavender) callout — e.g. "next steps" cards on setup,
   * "what to do now" panels. Not a semantic warning; just visual
   * emphasis to mark the next operator action. */
  background: var(--accent-subtle);
  border-color: var(--accent-default);
  border-left-width: 4px;
}

/* RFC 064 (v0.46.0) — Empty-state primitive.
 * Replaces the per-page `<p class="muted">No X yet.</p>` pattern. The
 * dashed border + tinted background distinguishes "this section is a
 * placeholder" from "this section has muted-coloured content."
 * Compact variant is for use inside a table cell or other narrow
 * context where the full padding would look ridiculous. */
.empty-state {
  background: var(--surface-subtle);
  border: var(--border-width-default) dashed var(--border-muted);
  border-radius: var(--radius-md);
  padding: var(--space-5);
  text-align: center;
  color: var(--fg-muted);
}
.empty-state--compact {
  padding: var(--space-3);
  border-style: solid;
  text-align: left;
}
.empty-state__message {
  font-size: var(--font-size-body);
  margin: 0 0 var(--space-2) 0;
  color: var(--fg-default);
}
.empty-state__hint {
  font-size: var(--font-size-caption);
  margin: 0 0 var(--space-3) 0;
}
.empty-state__action {
  display: inline-block;
}

/* ── Callout (RFC-MI-011, v0.50.1) ───────────────────────────────────── */
/* Persistent explanatory block. Not a flash banner (transient) and       */
/* not a card (no shadow / elevation). Used for setup instructions,        */
/* security-policy notes, and any "read this before you proceed" block.   */
/*                                                                         */
/* .callout           — neutral (surface-subtle + border-muted)           */
/* .callout--info     — informational (info-subtle + info-default border) */
/* .callout--success  — confirmation (success-subtle + success-default)   */
/* .callout--warning  — caution (warning-subtle + warning-default)        */
/* .callout--danger   — destructive action (danger-subtle + danger-default)*/
/*                                                                         */
/* Note: .card--callout (existing) uses accent fill for "next steps" CTA  */
/* blocks. The new .callout is tone-neutral; prefer it for read-only       */
/* informational copy.                                                     */
.callout {
  background: var(--surface-subtle);
  border: var(--border-width-default) solid var(--border-muted);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  color: var(--fg-default);
}
.callout + .callout { margin-top: var(--space-2); }
.callout--info {
  background: var(--info-subtle);
  border-color: var(--info-default);
}
.callout--success {
  background: var(--success-subtle);
  border-color: var(--success-default);
}
.callout--warning {
  background: var(--warning-subtle);
  border-color: var(--warning-default);
}
.callout--danger {
  background: var(--danger-subtle);
  border-color: var(--danger-default);
}
/* Heading inside a callout — body-size, no extra margin. */
.callout__title {
  font-size: var(--font-size-body);
  font-weight: var(--font-weight-medium);
  margin: 0 0 var(--space-2);
}

/* ── Card width variant (RFC-MI-041, v0.53.0) ───────────────────────── */
/* .card--narrow constrains the card to the content-narrow width,        */
/* used for the password-change form on /me/security/password and any    */
/* other isolated single-action form that should not stretch wide.       */
.card--narrow {
  max-width: var(--content-narrow-width);
}

"#;
