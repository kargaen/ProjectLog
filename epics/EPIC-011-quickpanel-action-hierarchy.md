# EPIC-011: QuickPanel action hierarchy and destructive-action safeguards

**Status:** closed
**Created:** 2026-07-27
**Architecture baseline:** 9197163

---

## 1. BDD — User Flows

### Flow 1: Timesheet shortcuts lead the action area

```gherkin
Given the QuickPanel is open in normal mode
When the user reaches the action area
Then “Yesterday + Today” and “Full Timesheet” are visible without opening another control
And both shortcuts have the same primary visual treatment
And “Yesterday + Today” precedes “Full Timesheet”
And About and reset actions do not compete with those shortcuts in the action area
```

### Flow 2: Secondary actions live in an accessible overflow menu

```gherkin
Given the QuickPanel is open in normal mode
When the user opens the “⋮ More” action overflow menu
Then “About,” “Reset Timesheet,” and “Reset Projects” are available
And the reset actions are visually separated from About
And both reset actions have a destructive visual treatment
And the menu can be operated by keyboard and dismissed without choosing an action
```

### Flow 3: Reset Timesheet requires an explicit decision

```gherkin
Given the action overflow menu is open
When the user chooses “Reset Timesheet”
Then the confirmation says “Permanently erase all timesheet data?”
And the reset does not execute before affirmative confirmation
And cancelling returns to the QuickPanel with the timesheet unchanged
And confirming executes the existing Reset Timesheet action exactly once
```

### Flow 4: Reset Projects requires an explicit decision

```gherkin
Given the action overflow menu is open
When the user chooses “Reset Projects”
Then the confirmation says “Permanently erase all saved projects?”
And the reset does not execute before affirmative confirmation
And cancelling returns to the QuickPanel with the saved projects unchanged
And confirming executes the existing Reset Projects action exactly once
```

**Out of scope for this epic:**
- Moving Reset Projects to a new Settings screen. ProjectLog currently exposes settings inline
  in the QuickPanel rather than through a separate screen, and there is no usage evidence that
  justifies adding another navigation step. The action remains in overflow for this slice.
- Changing the tray menu, compact-mode controls, reset semantics, or the data erased by either
  existing reset command.
- Adding analytics to measure Reset Projects usage; ProjectLog's local-only posture should not
  be expanded to settle a control-placement question.
- Reworking comment, project-entry, log-file, or settings controls outside the action hierarchy.

---

## 2. Function Call Signatures

*(deferred to revision 2)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

The accepted flows in §1 pin the product-specific labels, order, grouping, confirmation copy,
and exact-once execution behavior. The existing QuickPanel output at architecture baseline
`9197163` pins the callback contracts and the successful effects of both reset actions; this
epic changes how users reach and confirm those callbacks, not what the callbacks erase.

Keyboard behavior and accessibility semantics are pinned by the WAI-ARIA Authoring Practices
Menu Button Pattern: the trigger exposes its menu relationship and expanded state, opening
moves interaction into the menu, Escape closes it, and menu-item activation closes the menu.
The confirmation interaction is pinned by the WHATWG HTML Standard `confirm(message)`
algorithm: the supplied consequence message is presented before execution and the reset
proceeds only when the user gives an affirmative response.

### Test map

| Flow | Function call | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| 1 | QuickPanel action-area rendering | §1 Flow 1 accepted output | Existing mocked QuickPanel boot in `tests/ui/app.spec.ts` | Exact visible labels, DOM order, and shared primary class |
| 2 | “⋮ More” overflow trigger and menu keyboard interaction | WAI-ARIA Menu Button Pattern plus §1 Flow 2 | Existing mocked QuickPanel boot in `tests/ui/app.spec.ts` | Exact “⋮ More” accessible name, expanded state, grouping, and Escape dismissal |
| 2 | Computed overflow reset-action styles | §1 Flow 2 accepted output and the shipped QuickPanel palette | Chromium-rendered menu in `tests/ui/app.spec.ts` | Exact shared destructive class; reset foreground differs from About |
| 3 | `resetTimesheet()` reached through the overflow menu | WHATWG `confirm(message)`, baseline `9197163` reset callback, and §1 Flow 3 | Playwright dialog handling and mocked `reset_timesheet` invocation in `tests/ui/app.spec.ts` | Zero invocations on cancel; exactly one on confirm; exact “Permanently erase all timesheet data?” text |
| 4 | `resetProjects()` reached through the overflow menu | WHATWG `confirm(message)`, baseline `9197163` reset callback, and §1 Flow 4 | Playwright dialog handling and mocked `reset_projects` invocation in `tests/ui/app.spec.ts` | Zero invocations on cancel; exactly one on confirm; exact “Permanently erase all saved projects?” text |

### What is deliberately not tested

Pixel-perfect menu coordinates, shadows, animation timing, platform-native confirmation chrome,
and pointer trajectories are not pinned. Existing tests remain responsible for the resulting
timesheet/project state and About dialog contents; this epic tests placement, semantics,
destructive styling, warning clarity, cancellation, and exact-once dispatch.

---

## 4. Checklist

```md
[x] 1. Add failing primary-action, overflow semantics, destructive grouping, confirmation-cancel, and confirmation-accept cases in `tests/ui/app.spec.ts`, using the existing `window.__TAURI_MOCK__.invokedCommands` history — done when each §3 browser assertion fails for the hierarchy or safety behavior it names while existing QuickPanel tests still run
[~] 2. ~~Expose reset-command invocation counts from the existing browser mock in `tests/ui/helpers/tauri.ts`.~~ — removed during review because baseline `9197163` already exposes `window.__TAURI_MOCK__.invokedCommands`; item 1 consumes that existing contract directly
[x] 3. Render “Yesterday + Today” and “Full Timesheet” as the ordered primary controls and move About and both reset actions into an accessible overflow menu in `src/views/components/quickpanel/QuickPanelControls.view.svelte` — done when the hierarchy, menu semantics, keyboard, dismissal, and callback assertions from item 1 pass
[x] 4. Add primary-action, overflow-menu, separator, focus, and destructive-action styles in `src/views/screens/quickpanel.css` — done when the computed-style and visual-grouping assertions from item 1 pass without changing compact mode
[x] 5. Replace the generic reset prompts with explicit consequence-first confirmation copy in `src/controllers/projects/createProjectActionsController.ts` — done when the cancel and exact-once confirmation assertions from item 1 pass for both reset actions
```

---

## 5. Summary

### Architecture impact

- [x] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections
- [ ] **Requires a Constitution change** — a human decision, blocks this epic until resolved

### North star deviation

North star: *“ProjectLog is a frictionless time-tracking companion: always there, never in
the way.”*

**No.** The two frequent timesheet destinations become easier to identify and remain one click
away, while rare destructive operations gain intentional friction only after the user selects
them. No logging path or reconstructed time record changes.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Resolved: keep Reset Projects in overflow because no dedicated Settings surface or usage evidence exists today; any later move requires a separately formulated epic | No | Closed with EPIC-011 |

### New capability

None. This reorganizes existing QuickPanel actions and strengthens the existing confirmation
guardrails without adding a domain capability.
