# EPIC-003: Timesheet range correctness

**Status:** draft
**Created:** 2026-07-12
**Architecture baseline:** 299b448

Source: migrated from `DEVNOTES.md` (deleted in the same commit). Original text quoted per flow.

---

## 1. BDD — User Flows

### Flow 1: Yesterday + Today shows only yesterday and today

> DEVNOTES.md: "Yesterday+Today does not work, shows me the full"

```gherkin
Given the log contains entries spanning several weeks
When the user generates a Yesterday + Today timesheet
Then the preview and the exported file contain only yesterday's and today's entries
```

### Flow 2: Full timesheet lands on the current week

> DEVNOTES.md: "Generating full should always land on the current week, both in preview and
> in the excel file."

```gherkin
Given the log contains entries spanning several weeks
When the user generates a Full timesheet
Then the preview opens positioned on the current week
And the exported Excel file presents the current week the same way
```

**Out of scope for this epic:**
- Rounding behavior — already covered by shipped fixes and existing Rust tests.
- Tray-surface concerns (EPIC-002).

---

## 2. Function Call Signatures

*(deferred to revision 2)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Authority | Fixture | Tolerance |
|---|---|---|---|
| 1, 2 | Legacy application output — the append-only log format (`architecture/description/03-domain-boundaries.md`) is the single source; expected totals are hand-computed from a fixed multi-week fixture log | New fixture: multi-week log file, to be created in slice 1 | Exact — hours match hand computation to the minute |

### What is deliberately not tested

Excel cell styling and column widths — only range membership and totals.

### Verification note

`CHANGELOG.md`'s Unreleased section claims "Fixed Yesterday + Today showing empty yesterday
column" and "Fixed Full timesheet showing stale content" — related but not identical to the
quoted complaints. Slice 1 must first reproduce (or fail to reproduce) each complaint
against current `dev` before any fix is written.

---

## 4. Checklist

```md
[ ] 1. Failing test: Yesterday+Today range on a multi-week fixture log returns only 2 days (Rust, timesheet_service) — done when it fails for the right reason, or is shown to pass (then Flow 1 is already fixed; record and move on)
[ ] 2. Fix the range filter if test 1 failed — done when test 1 passes
[ ] 3. Failing test: Full generation is positioned on the current week — done when it fails for the right reason, or passes (record)
[ ] 4. Fix if needed — done when test 3 passes
```

---

## 5. Summary

### Architecture impact

- [x] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections
- [ ] **Requires a Constitution change**

### North star deviation

North star: "tell you exactly how much time you spend on what project several weeks back
with surgical accuracy."

**No — it is the north star.** Wrong ranges in the timesheet are the most direct possible
erosion of the reconstructed record's trustworthiness; this epic repairs it.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Are the two complaints already fixed by the Unreleased CHANGELOG entries? | No — the checklist's reproduce-first items resolve it mechanically | Slice 1 |

### New capability

None — bug fixes to existing behavior.
