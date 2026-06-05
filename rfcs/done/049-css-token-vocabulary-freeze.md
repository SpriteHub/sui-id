# RFC 049 — CSS token vocabulary freeze

**Status.** Implemented (v0.42.0)
**Priority.** P0 — blocker for v0.42.0
**Tracks.** UI/UX correctness baseline; Phase A of the
v0.42 → v1.0-rc plan.
**Touches.** `crates/sui-id-web/src/tokens.rs`,
`crates/sui-id-web/src/pages.rs`,
`crates/sui-id-web/src/components.rs`,
`.github/workflows/ci.yml`.

## Summary

Several `var(--…)` references in `pages.rs` and
`components.rs` point at CSS custom properties that are not
declared in `tokens.rs`. Browsers silently drop a declaration
whose `var()` does not resolve, so the affected DOM nodes
render with no border, no colour, no spacing — whichever
property was downgraded. This RFC renames every offending
reference to the canonical token name and adds a CI grep
that fails when any `var(--…)` references a token not
declared in `tokens.rs`.

## Background

The token vocabulary defined in
`crates/sui-id-web/src/tokens.rs` uses these prefixes:

- `--surface-*` — page surfaces
- `--fg-*` — foreground text
- `--accent-*` — accent (lavender) shades
- `--border-*` — border colours and widths
- `--state-*` — interaction states (hover, focus, active)
- `--danger-*`, `--warning-*`, `--success-*`, `--info-*` —
  semantic colours
- `--space-1` … `--space-6` — numeric spacing scale
- `--font-*`, `--line-height-*`, `--radius-*` — typographic
  and shape tokens

Inside `pages.rs`, inline `style="…"` attributes (RFC 067
will address the inline-style discipline separately) reach
for variables with the wrong prefix:

| Used in code                      | Canonical token                 |
|-----------------------------------|---------------------------------|
| `var(--colour-warn)`              | `var(--warning-default)`        |
| `var(--color-border)`             | `var(--border-default)`         |
| `var(--color-focus-ring)`         | `var(--state-focus)`            |
| `var(--color-surface-raised)`     | `var(--surface-elevated)`       |
| `var(--color-text-primary)`       | `var(--fg-default)`             |
| `var(--color-text-secondary)`     | `var(--fg-muted)`               |
| `var(--space-sm)`                 | `var(--space-2)` *(see note)*   |

The "color" / "colour" inconsistency is the giveaway: the
project standardised on British spelling for variable names
("`--surface`-style names"), and someone authoring inline
styles reached for the American "color" prefix from muscle
memory.

**Note on `--space-sm`.** The mapping is a judgement call —
the spacing scale is numeric, so any named-size mapping
(`--space-sm`, `--space-md`, `--space-lg`) is a synonym for
some specific number. The mapping above picks `--space-2`
(`8px`) for `sm`; reviewers should sanity-check the visual
result. A separate alternative — adding named synonyms to
`tokens.rs` — is discussed under "Alternatives considered."

## Goals

1. Every existing `var(--…)` reference resolves against a
   token declared in `tokens.rs`.
2. The same regression — a typoed variable name in a future
   inline style — fails CI immediately.
3. The token names in `tokens.rs` stay normative: no new
   synonyms are added unless the design system explicitly
   sanctions them.

## Detailed design

### Part A — site fixes

Mechanical search-and-replace in `pages.rs` and
`components.rs` according to the table above.

Affected file count: 2. Affected sites: ~12 (some variables
are referenced more than once).

### Part B — CI invariant

Add a `css-token-resolve` step to `.github/workflows/ci.yml`.

```yaml
  css-token-resolve:
    name: CSS tokens — every var() resolves
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: All var(--…) references resolve in tokens.rs
        run: |
          set -e
          # 1. Extract declared variables from tokens.rs and components.rs
          #    (each declaration looks like:  --name: value;)
          declared=$(grep -hoE '^\s*--[a-z0-9-]+\s*:' \
                       crates/sui-id-web/src/tokens.rs \
                       crates/sui-id-web/src/components.rs \
                     | sed 's/^[[:space:]]*//; s/[[:space:]]*:.*$//' \
                     | sort -u)

          # 2. Extract used variables from anywhere in the workspace
          used=$(grep -rhoE 'var\(--[a-z0-9-]+' crates/ \
                   --include='*.rs' --include='*.css' \
                 | sed 's/^var(//' \
                 | sort -u)

          # 3. Compute the set difference: used minus declared.
          unresolved=$(comm -23 <(echo "$used") <(echo "$declared"))

          if [ -n "$unresolved" ]; then
            echo "::error::CSS variable used but not declared:"
            echo "$unresolved"
            exit 1
          fi
```

The grep extracts declarations and uses with `--name`
(without the trailing `:` or `(`) and uses `comm` to compute
the asymmetric difference. False-positive surface: zero
across the current codebase (after the Part A fixes land).

### Part C — declared but not used (informational only)

A second, **non-failing** check reports declared tokens that
nothing references. This is useful hygiene but not a
blocker — some tokens (`--z-toast`, `--motion-slow`) are
declared for future use. The job logs the list as a warning
annotation, not an error:

```yaml
      - name: Report unused tokens (advisory)
        run: |
          ...
          unused=$(comm -13 <(echo "$used") <(echo "$declared") || true)
          if [ -n "$unused" ]; then
            echo "::warning::Declared tokens with no references:"
            echo "$unused"
          fi
```

## Alternatives considered

### Add `--space-sm`, `--space-md`, `--space-lg` as synonyms

Lower noise at the call site (`var(--space-sm)` reads
better than `var(--space-2)`). The cost is a second naming
system that contributors must keep in sync; if the numeric
scale changes (e.g. switching to a 5-step scale), every
synonym needs re-pointing. Out of scope here — RFC 061
revisits the palette and can add semantic spacing aliases
deliberately if it concludes they help. For now, the numeric
scale is normative.

### Run a real CSS linter

`stylelint` would catch this and more, at the cost of an npm
dependency in CI. The project ships zero JS bundles; pulling
npm in for one lint job is out of proportion. The 25-line
shell grep covers what we need.

## Test plan

- Pre-fix: run the Part B grep manually; capture the seven
  unresolved names.
- Post-fix: grep returns empty; `cargo build --workspace`
  still passes (it always did — CSS errors don't fail Rust
  compilation). Manual visual smoke: open the dashboard, look
  at the "Action required" card (which referenced
  `--colour-warn`) and confirm the warn-coloured left border
  now renders.
- CI: deliberately add `var(--colour-fake)` in a draft
  commit, confirm the `css-token-resolve` job fails. Drop
  the draft commit.

## Security considerations

None.

## Migration risk

None. CSS-only fix. Visual diff is a strict improvement
(previously-invisible warning borders, focus rings, and
elevated-surface backgrounds now appear).

## Estimated effort

~30 minutes for the search-and-replace, ~30 minutes for the
CI snippet, ~30 minutes for visual smoke + PR write-up.
**~1.5 hours total.**

## Version impact

Patch bump candidate, but bundled into v0.42.0 with the rest
of Phase A.
