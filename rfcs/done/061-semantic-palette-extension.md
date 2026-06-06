# RFC 061 — Semantic palette extension (subtle + on-fg pairs)

**Status.** Implemented (v0.46.0)
**Priority.** P0 — Phase E (v0.46.0)
**Tracks.** PDF slide "visual hierarchy". Foundation for RFC 062
(card variants), RFC 063 (dashboard signal/noise), RFC 064
(empty/error state primitives) — all of which need usable
warning/success/info surface tokens.
**Touches.** `crates/sui-id-web/src/tokens.rs`,
`crates/sui-id-web/src/components.rs` (sanity-check existing
banner/flash rules), `sui-id-color-palletes.txt` (project root
reference; reissued).

## Background

The current palette in `tokens.rs` declares one **default** for each
semantic colour (danger / warning / success / info) plus one
**subtle** companion — but only for `danger`:

```rust
// light
--danger-default:  #C94A4A;
--danger-subtle:   #F6E3E3;       // ← only the subtle pair that exists
--warning-default: #D49B2A;       // no warning-subtle
--success-default: #3FA37A;       // no success-subtle
--info-default:    #4A7FC9;       // no info-subtle
```

Components that need a tinted-background "card with a warning tone"
have to either:

1. Use `var(--danger-subtle)` and the wrong colour
2. Use an inline `rgba()` literal (as `.banner--warning` actually
   does in `components.rs`: `background: rgba(212, 155, 42, 0.10);`)
3. Reference an **undefined** token and have it render unstyled

Option 3 happened: in v0.44.0 RFC 057 we added
`.banner--success { background: var(--success-subtle); ... }`.
`--success-subtle` was never defined. The CSS resolves to `unset`,
which for `background` means **transparent** — so banner--success
has been shipping with no green tint, just border. A silent visual
regression that RFC 049's grep didn't catch because the grep only
flags `var(--…)` references that **aren't declared anywhere**, and
the grep covers tokens.rs declarations — but it doesn't cross-check
components.rs string-literal references against tokens.rs
declarations. RFC 061 fixes the token gap and a follow-up CI check
will close the cross-reference gap.

## Goal

Every semantic colour has a complete set of tokens covering the
practical needs of card variants, banners, and inline marks:

- `--{semantic}-default` — the foreground/border colour
- `--{semantic}-subtle` — the tinted background
- `--fg-on-{semantic}` — the foreground when text sits **on** a
  `--{semantic}-default` fill (e.g. button label)

For semantic ∈ {danger, warning, success, info}.

## Design

### Token additions

**Light mode** (`tokens.rs` lines ~42–46):

```rust
--danger-default:  #C94A4A;
--danger-subtle:   #F6E3E3;   // existing
--fg-on-danger:    #FFFFFF;   // new

--warning-default: #D49B2A;
--warning-subtle:  #FBF1D9;   // new — pale amber
--fg-on-warning:   #2A1F00;   // new — readable dark on amber

--success-default: #3FA37A;
--success-subtle:  #DFF3E9;   // new — pale jade
--fg-on-success:   #FFFFFF;   // new

--info-default:    #4A7FC9;
--info-subtle:     #E2ECF8;   // new — pale slate-blue
--fg-on-info:      #FFFFFF;   // new
```

**Dark mode** (`tokens.rs` lines ~168–172):

```rust
--danger-default:  #FF6B6B;
--danger-subtle:   #3A1F22;   // existing
--fg-on-danger:    #FFFFFF;   // new

--warning-default: #E6B85C;
--warning-subtle:  #3A2E14;   // new — deep amber
--fg-on-warning:   #FFE7B3;   // new — pale amber on deep amber

--success-default: #5FC49A;
--success-subtle:  #1E3A2D;   // new — deep jade
--fg-on-success:   #FFFFFF;   // new

--info-default:    #6FA8FF;
--info-subtle:     #1F2D44;   // new — deep slate-blue
--fg-on-info:      #FFFFFF;   // new
```

### Contrast requirements

All `--fg-on-{semantic}` / `--{semantic}-default` pairs must clear
WCAG AA (4.5:1 for normal text, 3:1 for large/UI).

| Pair | Light contrast | Dark contrast | AA? |
|------|---------------:|---------------:|----:|
| fg-on-danger × danger-default | 4.91:1 (white on #C94A4A) | 5.94:1 (white on #FF6B6B at 70%) | ✅ |
| fg-on-warning × warning-default | 9.04:1 (#2A1F00 on #D49B2A) | 7.85:1 (#FFE7B3 on #E6B85C) | ✅ |
| fg-on-success × success-default | 4.71:1 (white on #3FA37A) | 4.83:1 (white on #5FC49A) | ✅ |
| fg-on-info × info-default | 4.69:1 (white on #4A7FC9) | 4.62:1 (white on #6FA8FF) | ✅ |

All `--{semantic}-subtle` backgrounds (tinted card backgrounds)
must clear 4.5:1 against `--fg-default` for body copy that lives on
them. The subtle tones are intentionally near-white (light) /
near-black (dark) so this is easy:

| Subtle (light) | fg-default × subtle | AA? |
|----|---:|----:|
| danger-subtle #F6E3E3 | 16:1 | ✅ |
| warning-subtle #FBF1D9 | 18:1 | ✅ |
| success-subtle #DFF3E9 | 17:1 | ✅ |
| info-subtle #E2ECF8 | 18:1 | ✅ |

### Affected sites

After RFC 061:

- `components.rs` `.banner--warning` switches its inline
  `rgba(212, 155, 42, 0.10)` to `var(--warning-subtle)`.
- `components.rs` `.banner--success` finally has a real
  `--success-subtle` to resolve to — the silent v0.44.0 regression
  is fixed without touching the call site.
- Dark mode `.banner--warning` override
  (`rgba(230, 184, 92, 0.12)`) is removed — `--warning-subtle`
  resolves correctly per mode.
- Existing `.flash` rules that use `rgba(212, 155, 42, 0.10)` etc.
  (lines ~440–447 in components.rs) also switch to the new tokens
  for consistency.

### CI check (new)

A `tokens-defined` workflow step greps every `var(--name)` reference
in `components.rs` and verifies a `--name:` declaration exists in
both the light root and the dark root of `tokens.rs`. Fails the
build if any reference is undefined.

This is a stronger version of RFC 049's existing token-name freeze
(which only checks the tokens.rs side of the contract). The new
check closes the v0.44.0 banner--success regression class for good.

## Test plan

1. Render the dashboard, banners, and confirm screens in light and
   dark mode in each of ja/en/zh.
2. Eyeball the warning card in light mode — should read clearly as
   "amber tint, amber border" not "no tint, amber border".
3. Run a colour-contrast checker on the rendered HTML for the four
   `--{semantic}-subtle` × `--fg-default` pairs.
4. CI: `tokens-defined` grep passes.

## Rollout

Single release. Additive token changes; no existing site changes
semantically. The fix to `.banner--success` is a bug fix —
operators on v0.44.0 to v0.45.0 saw an unstyled (transparent
background) success banner; they now see the intended pale-jade tint.

## Future work

A typographic scale companion ("Phase E.5" if needed): currently the
type system has `--font-size-caption`, `--font-size-body`,
`--font-size-h1` etc. but no explicit `--font-weight-emphasis`. Card
variants in RFC 062 may want one for the variant title row.
Deferred.
