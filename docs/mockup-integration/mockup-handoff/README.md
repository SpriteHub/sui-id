**Package contents** (2,665 lines total):

| Document | Lines | Purpose |
| --- | --- | --- |
| **HANDOFF.md** | 986 | Primary explanatory document — 16 sections following the requested structure |
| **SCREEN_INVENTORY.md** | 330 | All 35 routes, role + primary action + shell + trait calls |
| **FLOW_SUMMARY.md** | 399 | The 5 user flows with ASCII sequence diagrams + UX intent |
| **OPEN_ISSUES.md** | 404 | 12 issues triaged by priority, each with decision owner |
| **IMPLEMENTATION_NOTES.md** | 546 | Practical integration guide — trait extensions, AppState wiring, code patterns |

**What the package answers**, per the instruction's §19 final-expectation list:

| Question the implementation team should be able to answer | Where |
| --- | --- |
| What is this mockup supposed to achieve? | HANDOFF §1, §3 |
| What experience should be preserved? | HANDOFF §3.3 (ABDD + minimalism), §4, §5 |
| Which parts are essential? | HANDOFF §3.3, §11.3 (step-up coverage), §12.1 (confirmed constraints) |
| Which parts are flexible? | HANDOFF §2.3 (treat-as table), §12 |
| Which parts are unresolved? | HANDOFF §12.2, OPEN_ISSUES |
| What to do if mockup and implementation differ? | HANDOFF §13, §14 |
| Who should be consulted when ambiguity remains? | OPEN_ISSUES (every issue has decision owner), HANDOFF §14.2 |

**Key design decisions documented as preserved/non-negotiable**:

- ABDD (Accessible by Default and by Design) — §10 with semantic-HTML / keyboard / screen-reader / never-colour-only rules
- Minimalism (one place, one thing) — §3.3.2, applied to setup wizard, settings tabs, danger zones, step-up route
- Step-up + impact summary as the **only** path for destructive actions — §5.5, §11.3
- Anti-enumeration wording rules — §11.1
- Fail-closed behaviours (HIBP, audit, key publish) — §11.2
- SSR-first, no-JS baseline — IMPLEMENTATION_NOTES §11

**Critical guidance for the team**:

- HANDOFF §14.3 — resolution priority order: security → robustness → maintainability → standards → usability → visual
- HANDOFF §14.4 — temporary-implementation rule when blocked
- HANDOFF §15.1 — week-by-week integration order minimising risk
- IMPLEMENTATION_NOTES §15 — "things to do early / defer / don't do at all"
