# EPIC-010: Current-week timesheet selection

**Status:** closed
**Created:** 2026-07-21
**Architecture baseline:** 949eec1

---

## 1. BDD — User Flows

### Flow 1: Full preview opens on the current week

```gherkin
Given the full timesheet contains the current week and one or more earlier weeks
When the user opens the Full timesheet preview
Then the current-week tab is selected
And the current week's rows are displayed
```

### Flow 2: Full export opens on the current week

```gherkin
Given the full timesheet contains the current week and one or more earlier weeks
When the user exports the Full timesheet to Excel
Then the workbook's active worksheet is the current-week worksheet
And every generated weekly worksheet remains present
```

### Flow 3: A missing current week falls back safely

```gherkin
Given the full timesheet has historical weeks but no recorded hours in the current week
When the user opens the preview or exports the workbook
Then the newest available week is selected or active
```

### Flow 4: Refresh preserves the reviewed week

```gherkin
Given the user selected a historical week in an open Full timesheet preview
When the user refreshes the preview and that week is still present
Then the same historical week remains selected
```

**Out of scope for this epic:**
- Timesheet range filtering and Yesterday + Today behavior, which already have regression coverage.
- Creating an empty current-week sheet when no current-week hours exist; the fallback uses the
  newest generated week without changing timesheet data.
- Changing worksheet order, worksheet names, exported values, or visual tab styling.
- Preserving a manually selected historical preview tab across closing and reopening the window;
  a newly opened Full timesheet starts on the current week.

---

## 2. Function Call Signatures

*(deferred to revision 2)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

The archived legacy requirement in `DEVNOTES.md` at commit `6a3e549` states: "Generating full
should always land on the current week, both in preview and in the excel file." That output
contract pins Flows 1 and 2. Flow 3 is pinned by the existing timesheet contract: full previews
contain only weeks reconstructed from persisted log entries, ordered chronologically. Flow 4
is pinned by the shipped `updateSheetIndex` behavior in
`src/controllers/timesheets/createTimesheetPreviewController.svelte.ts`, which preserves the
displayed sheet by name when it remains in a refreshed Full preview.

Both surfaces consume the same relative-week fixture contract in
`tests/timesheet-current-week.json`. The fixture describes work sessions as offsets
from the runtime current ISO week, allowing the Playwright mock and Rust log writer to derive
the same earlier/current/missing-current cases without freezing the calendar date.

### Test map

| Flow | Function call | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| Test boundary for 1, 3, 4 | `installTauriMocks(page, state, options)` | Playwright's single-argument `page.addInitScript` contract | Inline two-sheet preview bootstrap in `tests/ui/app.spec.ts` | Exact bootstrap title and sheet names |
| 1, 3, 4 | Full-preview open and refresh through the mocked Tauri bridge | Archived `DEVNOTES.md` output, the chronological sheet contract, and the shipped `updateSheetIndex` refresh behavior | Shared relative-week cases in `tests/timesheet-current-week.json`, loaded by `tests/ui/app.spec.ts` | Exact selected tab name after open and refresh |
| 2, 3 | `generate(data_dir, TimesheetOptions::full(TimesheetRange::All))` | Archived `DEVNOTES.md` output plus `rust_xlsxwriter`'s active-worksheet contract | The same shared relative-week cases in `tests/timesheet-current-week.json`, converted to log entries by `src-tauri/src/timesheet.rs` | Exact active worksheet name; exact worksheet count |

### What is deliberately not tested

Tab colors, workbook theme/styling, Excel's window layout, and the ordering or values already
covered by existing timesheet tests are not retested. The new assertions cover only initial
selection, refresh preservation, active worksheet, fallback, and preservation of all weekly
sheets.

---

## 4. Checklist

```md
[x] 1. Add the earlier/current/missing-current relative-week cases in `tests/timesheet-current-week.json` — done when the fixture represents every case named in the §3 test map without a fixed calendar date
[x] 1a. (added 2026-07-21) Pass the Tauri mock state and preview options through one serialized Playwright init argument in `tests/ui/helpers/tauri.ts` — done when the focused custom-preview bootstrap regression in `tests/ui/app.spec.ts` passes
[x] 2. Add failing Full-preview current-week, newest-fallback, and refresh-preservation cases in `tests/ui/app.spec.ts` using item 1 — done when the open cases fail because the first sheet is selected and the refresh case pins the selected historical sheet
[x] 3. Select the runtime current ISO-week sheet on initial Full-preview load, fall back to the newest sheet, and preserve an existing selection during refresh in `src/controllers/timesheets/createTimesheetPreviewController.svelte.ts` — done when item 2 passes
[x] 4. Add failing exported-workbook active-sheet and newest-fallback cases in `src-tauri/src/timesheet.rs` using item 1 — done when the generated workbook contains every expected week but opens on the wrong worksheet
[x] 5. Mark the runtime current ISO-week worksheet active, falling back to the newest worksheet, in `src-tauri/src/services/export_service.rs` — done when item 4 and the existing Rust timesheet suite pass
```

---

## 5. Summary

### Architecture impact

- [x] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections
- [ ] **Requires a Constitution change** — a human decision, blocks this epic until resolved

### North star deviation

North star: *"ProjectLog can tell you, with surgical accuracy, exactly how much time you spent
on what, weeks back."*

**No.** The epic changes only which already-generated week is presented first. It preserves
the complete reconstructed history and makes the most immediately relevant week visible in
both review surfaces without altering logged time or reported totals.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| — | None | No | — |

### New capability

None. This restores the archived presentation behavior of the existing full-timesheet preview
and Excel export.
