# EPIC-005: Logged-time test depth

**Status:** draft
**Created:** 2026-07-12
**Architecture baseline:** 299b448

Source: migrated from `DEVNOTES.md` (deleted in the same commit). Original text quoted below.

---

## 1. BDD — User Flows

### Flow 1: Escaped-bug analysis

> DEVNOTES.md: "Investigate why the following bugs were not caught in the tests (not the
> actual feedback/feature requests)" — referring to the bugs now tracked as EPIC-002 Q1,
> EPIC-003, and EPIC-004.

```gherkin
Given the list of bugs that shipped despite the test suite
When each is traced to the test that should have caught it
Then each gap is named: missing fixture, missing assertion, or untestable layer
And each gap becomes a checklist item here or a note in the owning epic
```

### Flow 2: Long-range logged-time fixtures

> DEVNOTES.md: "The test for logged time should be longer and more detailed, so the ui/code
> tests go into edge cases and test on a longer set of data."

```gherkin
Given a fixture log spanning multiple weeks with edge cases (overnight sessions, zero-length
  transitions, comments, ad-hoc projects, week boundaries)
When the timesheet suite runs against it
Then per-day and per-project totals match hand-computed values exactly
```

**Out of scope for this epic:**
- Fixing the bugs themselves — EPIC-002/003/004 own those.
- E2E testing of native tray/dialog surfaces (unreachable by the current harness; see
  drift-audit Finding 9's open decision on test layout).

---

## 2. Function Call Signatures

*(deferred to revision 2 — this epic mostly adds tests, not contracts)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Authority | Fixture | Tolerance |
|---|---|---|---|
| 2 | Hand-computed totals over a versioned fixture log (the log format spec in `architecture/description/03-domain-boundaries.md` defines parsing) | New multi-week fixture, shared with EPIC-003 | Exact to the minute |

### What is deliberately not tested

Performance on very large logs — correctness only, until a size problem is observed.

---

## 4. Checklist

```md
[ ] 1. Write the escaped-bug analysis (one paragraph per bug: which test should have caught it, why it didn't) into this epic — done when each of the four DEVNOTES bugs has a named gap
[ ] 2. Create the multi-week edge-case fixture log with hand-computed expected totals — done when EPIC-003 slice 1 can consume it
[ ] 3. Extend the Rust timesheet tests to run against the fixture — done when they pass and cover week-boundary + overnight cases that previously had no assertion
[ ] 4. Extend `tests/ui/app.spec.ts` scenarios where item 1 found UI-layer gaps — done when the named gaps have failing-then-passing tests
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [x] Amends Description sections: possibly `constitution/09-testing-philosophy.md`'s critical-regression list — **that is Constitution-class, so any change there is a human decision, not part of this epic's automatic scope**
- [ ] **Requires a Constitution change** — only if the regression list must grow; flagged, not assumed

### North star deviation

North star: "unshakeable trust in what gets reconstructed."

**No — it fortifies it.** Deeper fixtures are how the surgical-accuracy claim stays true.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Test directory layout mismatch (docs say `tests/e2e/` + `tests/rust/`; code has `tests/ui/` + inline Rust tests) — drift-audit Finding 9 | No — new tests follow the existing layout until the finding is decided | Before restructuring anything |

### New capability

None — test infrastructure only.
