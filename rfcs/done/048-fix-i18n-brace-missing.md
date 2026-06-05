# RFC 048 — Fix `t.xxx` brace-missing literals in `pages.rs`

**Status.** Implemented (v0.42.0)
**Priority.** P0 — blocker for v0.42.0
**Tracks.** UI/UX correctness baseline; Phase A of the
v0.42 → v1.0-rc plan.
**Touches.** `crates/sui-id-web/src/pages.rs`,
`.github/workflows/ci.yml`.

## Summary

The Leptos `view!` macro treats bare identifiers between tags
as **text content**, not expressions. Forty-eight call sites
in `pages.rs` omit the curly braces required for value
interpolation, so on the rendered page the visitor sees the
literal source text `t.dashboard_title` (and 47 others) where
a localised heading or button label should appear. This RFC
fixes every site and adds a CI grep that fails when the
pattern recurs.

## Background

The `Strings` table in `sui-id-i18n` is consumed via a typed
borrow:

```rust
let t = lang.strings();
view! { <h1>{t.dashboard_title}</h1> }  // correct
view! { <h1>t.dashboard_title</h1>   }  // BUG: renders "t.dashboard_title"
```

Both forms compile. The first interpolates the `&'static str`
field. The second is a Leptos macro-time text literal — the
characters `t`, `.`, `d`, `a`, `s`, … land in the DOM
verbatim.

Forty-eight current call sites use the buggy form:

```
$ grep -cE '">t\.[a-z_]+</' crates/sui-id-web/src/pages.rs
48
```

The affected sites include `<h1>` page titles, primary-action
button labels, badge text, and section headings on the
dashboard, users, clients, audit, signing-keys and settings
pages. None are edge cases. The RFCs that supposedly
delivered these pages (029, 035, 039, 040, 041, 042, 043,
046) all shipped with the defect because there is no CI
check for it.

## Goals

1. Fix every brace-missing site.
2. Make the same regression undetectable-by-eye — and
   guaranteed to fail in CI on the next PR.
3. Land in one commit; not split across follow-ups.

## Detailed design

### Part A — the 48 site fixes

Mechanical. Every line matching `">t\.[a-z_]+</'` wraps the
expression in curly braces:

```rust
// before:
<h1 class="page-header__title">t.dashboard_title</h1>
// after:
<h1 class="page-header__title">{t.dashboard_title}</h1>
```

The same change applies whether the parent is `<h1>`, `<h2>`,
`<h3>`, `<span>`, `<button>`, `<strong>`, `<td>`, or any
other element. No other change is needed at the call site.

For audit, the file is grepped before and after:

```
$ grep -nE '">t\.[a-z_]+</' crates/sui-id-web/src/pages.rs | wc -l
# Expected before: 48
# Expected after:   0
```

A list of all 48 line numbers is captured in the PR
description so reviewers can map "before" to "after" without
re-running the grep.

### Part B — CI invariant

A new `lint-text-leaks` step is added to
`.github/workflows/ci.yml`. It is fast, dependency-free, and
runs before the build step so the failure surfaces early.

```yaml
  text-leaks:
    name: text-leak invariants
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: No bare i18n expressions in view! children
        run: |
          set -e
          found=$(grep -rEn '">t\.[a-z_]+</' crates/ \
                    --include='*.rs' || true)
          if [ -n "$found" ]; then
            echo "::error::Bare 't.field' identifier as view! child."
            echo "These render as literal text — wrap in {…}:"
            echo "$found"
            exit 1
          fi
```

The grep is intentionally narrow:
- Anchored on `">` (the closing of a tag's last attribute,
  or the `>` of a no-attribute tag).
- `t\.[a-z_]+` captures only the canonical `t.field_name`
  shape used by every translation call site in the codebase.
- Anchored on `</` (the open of the closing tag, so the
  bare identifier must be the entire element body — the
  same shape the bug always takes).

This is precise enough that it has zero false positives
across the current codebase, and broad enough that the next
forgotten brace fails CI immediately.

## Why not parse the macro instead?

A full Leptos macro analyser would catch more (including
`{t.foo}` typoed as `{ t.fooo }` against a missing field —
which the type system already catches). The grep above is
~10 lines of YAML and runs in 1 second on the full tree.
Building a heavier tool earns nothing beyond what the
compiler + this grep cover together.

## Test plan

- Pre-fix: run the grep manually, capture the 48 lines as a
  baseline.
- Post-fix: grep returns nothing; `cargo build --workspace`
  still passes; `cargo test --workspace` shows no behavioural
  diff (since the bug rendered literal text but did not
  change semantics).
- CI: deliberately reintroduce one bug in a draft commit;
  confirm the `text-leaks` job fails. Drop the draft commit.

E2E tests already exist for several of the affected pages
(dashboard, users, clients, audit). Their HTML-content
assertions can be tightened in a follow-up to assert the
translated text appears — but that is not in scope here,
because the existing tests pass against the buggy code (they
assert structural elements, not text content), and tightening
them is best done after RFC 050 (admin chrome i18n) lands so
the assertions have a stable Strings vocabulary.

## Security considerations

None. The rendered text was always whatever the source typed.
This is a display correctness fix, not a privilege boundary
change.

## Migration risk

None. SQL schema unchanged. No serialised state involved.
No API surface change.

## Estimated effort

~30 minutes for the 48 mechanical edits, ~30 minutes for the
CI snippet, ~30 minutes for the PR write-up listing each
fix. **~1.5 hours total.**

## Version impact

Patch bump candidate, but bundled into v0.42.0 with the rest
of Phase A.
