# Refactor Summary

## Outcome

The refactor moved ProjectLog meaningfully closer to the intended MVC architecture without forcing a risky full rewrite.

The codebase now has clearer architectural boundaries on both the native and frontend sides, stronger documentation, better workflow support, and a more useful automated test baseline. The app remains functional, and the current state is good enough to resume from later without having to reconstruct why changes were made.

## What was completed

### 1. Product and architecture documentation was clarified

- Added a focused `README.md` with the mission statement and product framing.
- Reworked `ARCHITECTURE.md` into the current strong source of truth for:
  - strict MVC thinking
  - "Two Surfaces, One Domain"
  - self-documenting folder structure
  - domain boundaries and transport rules
- Added `WORKFLOWS.md` as a reusable workflow template for tasks, versioning, release flow, WIP flow, tests, and CI/CD thinking.

This gives the project a much stronger foundation for future refactor work.

### 2. Native-side MVC extraction started successfully

The Rust/Tauri backend was split into more explicit command and controller responsibilities.

New command modules were introduced under `src-tauri/src/commands/`:

- `project_commands.rs`
- `settings_commands.rs`
- `shell_commands.rs`
- `timesheet_commands.rs`

New controller modules were introduced under `src-tauri/src/controllers/`:

- `project_controller.rs`
- `settings_controller.rs`
- `shell_controller.rs`
- `timesheet_controller.rs`

This is an important architectural step because it reduces pressure on `src-tauri/src/lib.rs` and separates transport concerns from business logic.

### 3. `src-tauri/src/lib.rs` became thinner, but is not finished

The composition root is improved, but it still carries too much responsibility.

It still appears to own or coordinate too much of the following:

- app setup
- lifecycle wiring
- some shared app state concerns
- native registration and orchestration

So the refactor direction is correct, but `lib.rs` is still a partial bottleneck and remains a major follow-up target.

### 4. Timesheet logic got better test protection

The timesheet domain received stronger Rust-side verification.

Notable improvements:

- relative-to-now fixture coverage was added for yesterday, today, and multi-week scenarios
- preview behavior was checked more realistically
- export-related behavior remained covered
- a deferred note was recorded that true clock injection should eventually replace relative fixtures for full determinism

This was the right tradeoff: exact timesheet correctness is better protected in Rust tests than in browser UI tests.

### 5. Playwright coverage was reshaped into a leaner, higher-value UI suite

The UI test approach improved substantially.

Instead of many shallow checks, the suite now focuses on a smaller number of stronger regression workflows.

Current coverage includes:

- QuickPanel shell rendering
- manual project selection workflow
- A-Z ordering workflow
- recent-mode workflow
- core QuickPanel actions
- full timesheet preview rendering
- yesterday + today preview rendering
- preview actions for refresh, rounding, export, and close

This is a much better baseline for future UI expansion.

### 6. UI test data is now isolated from real user data

The browser-based Tauri mocks were improved so tests do not depend on whatever the user currently has in their real project list.

The mocked project list now includes richer stable sample data with:

- alphabetic edge cases
- long names
- special characters
- ordering edge cases
- enough variety to expose sorting and selector problems

This makes the UI suite more trustworthy during future debugging and refactors.

### 7. Debugging workflows for UI testing were improved

Editor tasks were added or refined so Playwright can be run in multiple useful modes:

- normal full test run
- headed UI run
- debug-step style run

The debug-step flow was further improved so it can pause automatically between meaningful UI actions instead of requiring manual stepping through every action.

Headless execution remains unaffected and fast.

### 8. Frontend MVC extraction has now started properly

The frontend is no longer only documented as MVC-adjacent. It now has real controller and bridge seams.

New frontend structure was introduced:

- `src/models/types.ts` now owns the shared frontend domain types
- `src/services/bridge/quickPanelBridge.ts` and `src/services/bridge/timesheetBridge.ts` now own typed Tauri command transport
- `src/controllers/quickpanel/createQuickPanelController.svelte.ts` now owns QuickPanel state, event wiring, settings persistence, update flow, and project interactions
- `src/controllers/timesheets/createTimesheetPreviewController.svelte.ts` now owns timesheet preview loading, refresh, rounding, export, and hover state
- `src/lib/types.ts` now acts as a compatibility re-export instead of remaining the canonical type home

The user-facing screens became meaningfully thinner:

- `src/App.svelte` is now mostly a screen shell that binds state and callbacks from the controller

This is an important checkpoint because it turns the frontend refactor from an architectural intention into an actual working pattern the rest of the UI can now follow.

### 9. QuickPanel view extraction is now underway

The frontend refactor is no longer only about moving logic out of Svelte files. The view layer itself has now started to take shape.

New view-layer structure was introduced:

- `src/views/screens/QuickPanelScreen.svelte` now owns QuickPanel screen composition
- `src/views/components/quickpanel/QuickPanelHeader.view.svelte`
- `src/views/components/quickpanel/QuickPanelControls.view.svelte`
- `src/views/components/quickpanel/CompactModeFooter.view.svelte`
- `src/views/components/projects/ProjectListPanel.view.svelte`
- `src/views/components/dialogs/InputDialog.view.svelte`
- `src/views/components/dialogs/AboutDialog.view.svelte`
- `src/views/components/dialogs/UpdateDialog.view.svelte`
- `src/views/screens/quickpanel.css` now centralizes the QuickPanel styling that used to live in `src/App.svelte`

This changed the role of `src/App.svelte` substantially:

- it is now a small composition/root-routing file rather than the main UI implementation
- QuickPanel rendering is delegated to a screen-level view
- the screen itself now composes passive view components instead of keeping every section inline

This is a meaningful architectural gain because the controller/view separation is now visible in the folder structure, not just present in the code flow.

### 10. Timesheet preview now follows the same `views/` architecture

The frontend no longer has one surface using the new architecture and one surface still living in a legacy component file.

New timesheet view-layer structure was introduced:

- `src/views/screens/TimesheetScreen.svelte` now owns timesheet preview screen composition
- `src/views/components/timesheets/TimesheetHeader.view.svelte`
- `src/views/components/timesheets/TimesheetTable.view.svelte`
- `src/views/components/timesheets/TimesheetFooter.view.svelte`
- `src/views/components/timesheets/TimesheetStatePanel.view.svelte`
- `src/views/screens/timesheet.css` now owns the timesheet preview styling

This also simplified the remaining app root responsibilities further:

- `src/App.svelte` now routes between QuickPanel and Timesheet screen components
- the legacy `src/lib/components/TimesheetPreviewWindow.svelte` file was removed
- the existing timesheet controller was preserved, so this change was mostly structural rather than behavioral

This is another meaningful checkpoint because both major frontend surfaces now use the same controller-to-screen-to-view flow.

### 11. QuickPanel controller responsibilities are now split into real action and state modules

The QuickPanel frontend logic is no longer concentrated in one giant controller file.

New QuickPanel controller-support modules were introduced under `src/controllers/quickpanel/`:

- `createQuickPanelProjectActions.ts`
- `createQuickPanelSettingsActions.ts`
- `createQuickPanelDialogActions.ts`
- `createQuickPanelUpdateActions.ts`
- `createQuickPanelShellActions.ts`
- `createQuickPanelStateSync.ts`
- `quickPanelTypes.ts`

This changed the role of `createQuickPanelController.svelte.ts` in an important way:

- it now behaves much more like a composition/orchestration layer
- project actions, settings actions, dialog behavior, update flow, and shell/window behavior now live behind clearer module boundaries
- state loading, ignored-event handling, manual-order syncing, and debounced UI-settings persistence now live in a dedicated state-sync module instead of staying inline in the controller

This is a meaningful follow-up to the earlier frontend MVC extraction because the controller pattern now exists at two levels:

- screens no longer own native command transport directly
- the main QuickPanel controller itself no longer owns every domain behavior inline

### 12. QuickPanel mount and event wiring now live in a dedicated lifecycle helper

The QuickPanel controller no longer owns all of its mount-time orchestration inline.

Another QuickPanel support module was introduced under `src/controllers/quickpanel/`:

- `createQuickPanelLifecycle.ts`

This changed the controller boundary again in a useful way:

- lifecycle startup, event listeners, bounds persistence polling, and cleanup are now grouped in one dedicated helper
- dialog prompt mode and close-on-submit behavior are now treated as part of controller state instead of living in loose local variables
- `createQuickPanelController.svelte.ts` is now much closer to a composition root that wires state sync, actions, shell behavior, update flow, and lifecycle together

This is a good architectural checkpoint because the remaining controller complexity is now much more intentional. What is left in the main controller is mostly dependency wiring and derived view state rather than mixed lifecycle and domain behavior.

## What worked well

- Using architecture documentation as the source of truth before pushing further refactor changes.
- Moving native logic into controllers without attempting to reorganize everything at once.
- Extracting frontend controller and bridge layers without destabilizing the existing QuickPanel or timesheet preview behavior.
- Moving QuickPanel rendering into a real `views/` tree without breaking the existing browser regression suite.
- Moving the timesheet preview surface into the same `views/` architecture without breaking its existing workflows.
- Continuing the QuickPanel controller refactor by extracting action and state-sync modules instead of letting the new controller become a second monolith.
- Continuing that same QuickPanel refactor by extracting lifecycle setup and listener cleanup into a dedicated helper instead of leaving mount-time orchestration inline.
- Keeping browser tests focused on user-visible workflows instead of overfitting to implementation detail.
- Using Rust tests for timesheet correctness and Playwright for interaction confidence.
- Using richer mock data to stabilize UI tests.

## What remains unfinished

### 1. Frontend MVC is still incomplete

The frontend is meaningfully closer to the target architecture described in `ARCHITECTURE.md`, but it is not finished yet.

Most notably:

- `src/App.svelte` is no longer the main frontend bottleneck, and the QuickPanel controller is now much better factored internally, but `src/views/screens/QuickPanelScreen.svelte` still carries a substantial amount of composition and QuickPanel-specific prop wiring
- `src/views/screens/TimesheetScreen.svelte` is smaller than the old implementation, but it still owns table-level composition and could be split further if that becomes valuable
- the QuickPanel controller now delegates to action, state-sync, and lifecycle modules, but its derived view state and screen-facing callback surface are still fairly broad
- passive view components now exist for both major frontend surfaces, but the view tree is still not yet aligned to all of the target domains from `ARCHITECTURE.md`
- dedicated stores or view-model boundaries are not in place yet, so controller-owned state is still doing most of the coordination work
- some frontend responsibilities are cleaner than before, but the UI surface is not yet fully decomposed into the documented MVC tree

This remains the next major refactor area, but it is now a continuation problem rather than a greenfield one.

### 2. Native composition and tray behavior likely still need cleanup

Even after controller extraction:

- `src-tauri/src/lib.rs` still appears too central
- `src-tauri/src/tray.rs` is likely still too smart
- shell-level coordination probably still needs a cleaner application-service or orchestration layer

### 3. Native-only behavior is not fully covered by browser tests

Playwright is now useful, but it still cannot fully validate:

- true tray behavior
- real native multi-window interactions
- OS-level opener integrations
- actual spawned preview-window behavior from the desktop shell

Deferred comments were added in the test suite to make this explicit.

### 4. Clock determinism is not solved yet

The timesheet tests are stronger, but they still rely on relative fixture strategies instead of full clock injection.

A future deterministic time abstraction would improve confidence and reduce edge-case brittleness.

### 5. Minor cleanup remains

There is still a warning in `src-tauri/src/controllers/shell_controller.rs` for unused imports:

- `Emitter`
- `Manager`

This is minor, but should be cleaned up later.

## Current validated state

At the end of this phase, the following validation passed:

- Rust tests: `29 passed`
- Frontend production build: `npm run build`
- Playwright UI tests: `8 passed`

This means the current checkpoint is stable enough to continue from later, and neither the controller/bridge extraction nor the new QuickPanel and Timesheet view decomposition broke the existing validated workflows. The known Rust warning in `shell_controller.rs` still remains, but it is unchanged from earlier checkpoints.

## Recommended next steps

When work resumes, the most sensible order is:

1. Continue the frontend refactor from the new controller/bridge baseline.
   - Continue decomposing `src/views/screens/QuickPanelScreen.svelte`
   - Decide whether QuickPanel update status, dialog rendering, or settings sections should become additional passive screen-level components
   - Decide whether the broad QuickPanel screen prop surface should be grouped into screen-specific view-model objects or section-level prop contracts
   - Decide whether QuickPanel subviews should be reorganized further into project/session/settings-specific folders
   - Decide whether `TimesheetScreen.svelte` should stay as the table-composition layer or be split further into smaller screen-level sections
   - Decide whether lightweight stores are now useful or whether controller-owned state should remain the primary pattern
   - Keep views passive and keep native command calls inside bridge modules

2. Continue thinning `src-tauri/src/lib.rs`.
   - Move orchestration/setup responsibilities into clearer modules where possible

3. Review `src-tauri/src/tray.rs`.
   - Ensure tray handling delegates quickly and does not become a second controller layer

4. Add a small native integration strategy for behavior browser Playwright cannot prove.
   - tray interactions
   - true preview window spawning
   - external opener flows

5. Consider introducing a clock abstraction for timesheet generation tests.

## Resume point

The refactor did not end in a half-broken state. It ended in a useful transitional state:

- architecture intent is much clearer
- backend boundaries are better than before
- frontend controller and bridge seams now exist and are working
- the QuickPanel controller itself now has clearer internal boundaries for actions, shell behavior, update flow, dialogs, state sync, and lifecycle wiring
- a real `views/` tree now exists for the QuickPanel surface
- the Timesheet preview surface now also lives inside the same `views/` architecture
- test coverage is stronger and more realistic
- the current UI behavior stayed green through the frontend architectural shift
- future work is easier to reason about

The next session should treat this as a guided continuation focused on deeper screen decomposition, state-boundary decisions, and remaining composition cleanup, not as a recovery task.
