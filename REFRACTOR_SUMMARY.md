# Refactor Work Breakdown Structure

## Purpose

This document is the execution checklist for finishing the ProjectLog refactor.

It replaces narrative progress notes with a complete work-breakdown structure. When every item in this file is done, the refactor is done.

---

## Done Rule

The refactor is complete when:

- every major responsibility lives in the layer described by `ARCHITECTURE.md`
- no root file, tray handler, Tauri command, or Svelte view is acting like a hidden controller
- duplicated logic across surfaces and layers has been removed or intentionally centralized
- native and frontend workflows are still covered by the appropriate tests

---

## Completion Checklist

### 1. Native Composition Root

- [x] `src-tauri/src/lib.rs` is only a composition root.
- [x] `lib.rs` only wires modules, plugins, commands, lifecycle hooks, and shared state exports.
- [x] Native setup responsibilities are fully delegated to dedicated setup modules.
- [x] Native lifecycle responsibilities are fully delegated to dedicated lifecycle modules.
- [x] No business workflow logic remains inline in `lib.rs`.

### 2. Native State and Shared DTO Boundaries

- [x] Shared native app state lives in dedicated state modules.
- [x] Shared DTO-like structs used across commands/controllers are defined in dedicated locations, not opportunistically inside root files.
- [x] State fields have clear ownership and are not acting as a dumping ground for unrelated behavior.
- [x] The native state structure matches the responsibilities described in `ARCHITECTURE.md`.

### 3. Native Commands

- [x] All Tauri commands are thin transport adapters.
- [x] Commands delegate immediately to controllers or services.
- [x] Commands do not own persistence, validation, orchestration, or shell logic.
- [ ] Command naming and file layout match the documented domain structure.

### 4. Native Controllers

- [x] Project workflows live in project-oriented controllers.
- [x] Settings workflows live in settings-oriented controllers.
- [x] Shell/window/tray-facing workflows live in shell-oriented controllers.
- [x] Timesheet workflows live in timesheet-oriented controllers.
- [x] Controllers own workflow decisions, but not raw transport plumbing.
- [x] Controllers do not duplicate each other’s logic across domains.

### 5. Native Services and Supporting Modules

- [ ] Domain services exist wherever controller logic is still too heavy or too mixed.
- [x] Shell-specific helpers are separated from project/session/timesheet domain behavior.
- [x] Window-specific behavior is isolated if it still clutters controllers.
- [x] Setup/bootstrap helpers are separated from runtime workflow helpers.
- [ ] Any remaining mixed-responsibility helpers are either split or clearly justified.

### 6. Tray Layer

- [ ] `src-tauri/src/tray.rs` is only an event-adapter layer.
- [x] Tray menu construction is separate from tray event handling where that improves clarity.
- [x] Tray handlers delegate quickly to controllers or shell helpers.
- [x] Tray handlers do not duplicate project, settings, timesheet, or shell workflows.
- [x] Tray-specific logic is limited to menu wiring, event decoding, and tray-surface concerns.
- [ ] The tray is no longer effectively a second controller layer.

### 7. Native Domain Separation

- [x] Project behavior is clearly separate from shell behavior.
- [x] Session/log behavior is clearly separate from shell behavior.
- [x] Timesheet behavior is clearly separate from shell behavior.
- [x] Settings behavior is clearly separate from shell behavior.
- [x] Diagnostics behavior is clearly separate from project/session/timesheet logic.
- [x] Cross-domain interactions happen through deliberate controller/service boundaries.

### 8. Frontend Root and Screen Composition

- [ ] `src/App.svelte` is only an app/root composition file.
- [ ] `src/views/screens/QuickPanelScreen.svelte` is primarily a screen composition layer.
- [ ] `src/views/screens/TimesheetScreen.svelte` is primarily a screen composition layer.
- [ ] Screen files do not accumulate controller logic, state orchestration, or native transport behavior.
- [ ] Remaining heavy screen-level composition is split further if it still obscures responsibilities.

### 9. Frontend Controllers

- [ ] Frontend business behavior lives in controllers, not in views.
- [ ] QuickPanel behavior is split into coherent controller modules with clear ownership.
- [ ] Timesheet preview behavior is split into coherent controller modules with clear ownership.
- [ ] Controller modules are organized by domain or responsibility, not by arbitrary convenience.
- [ ] Controllers own async coordination, derived UI behavior, and bridge usage.
- [ ] Controllers do not become new monoliths after extraction.

### 10. Frontend Views

- [ ] Views are passive: props in, callbacks out.
- [ ] Views do not call Tauri commands directly.
- [ ] Views do not contain hidden business rules.
- [ ] View decomposition matches meaningful UI or domain boundaries.
- [ ] Dialogs, settings sections, update sections, and project sections are split enough that responsibilities are obvious.
- [ ] The `views/` tree is close enough to the target architecture that file location explains responsibility.

### 11. Frontend Bridge Layer

- [ ] Bridge modules are the only frontend layer that talks to native commands.
- [ ] Bridge modules stay transport-focused and do not accumulate business rules.
- [ ] Bridge boundaries are explicit and easy to follow from controller to native command.
- [ ] Bridge naming and structure match the domains they serve.

### 12. Frontend State Boundaries

- [ ] Shared frontend state boundaries are intentional and stable.
- [ ] Any remaining broad prop surfaces are simplified or grouped into clearer view-model contracts.
- [ ] If stores are used, they have clear ownership and do not duplicate controller responsibilities.
- [ ] If controller-owned state remains the main pattern, that boundary is still clean and maintainable.
- [ ] No major frontend workflow depends on ad hoc state scattered across screens and views.

### 13. Type and Model Placement

- [ ] Shared frontend domain types live in canonical model/type locations.
- [ ] Shared native domain/state/DTO types live in canonical native locations.
- [ ] Compatibility re-exports are minimized or intentionally documented.
- [ ] Types are not stranded in legacy folders that no longer match the architecture.
- [ ] The folder tree is self-explanatory enough that a new contributor can infer where new code belongs.

### 14. Duplication Removal

- [x] Duplicate workflow logic between tray and commands has been removed.
- [x] Duplicate workflow logic between tray and controllers has been removed.
- [ ] Duplicate workflow logic between views and controllers has been removed.
- [ ] Duplicate workflow logic between screens and child views has been removed where it harms clarity.
- [x] Duplicate shell/open/file/prompt behavior has been centralized.
- [ ] Duplicate project/session/timesheet behavior has been centralized.

### 15. Architecture Alignment

- [ ] The implemented structure materially matches `ARCHITECTURE.md`.
- [ ] Any intentional deviations from `ARCHITECTURE.md` are small, explicit, and justified.
- [ ] The codebase communicates responsibilities by structure, not by tribal knowledge.
- [ ] New work can be placed confidently without reopening architectural uncertainty.

### 16. Test Coverage and Verification

- [x] Rust tests still cover timesheet/domain correctness.
- [ ] Playwright still covers key user workflows.
- [ ] Native refactor changes that browser tests cannot prove have an explicit verification strategy.
- [ ] No critical workflow has silently lost coverage during refactor cleanup.
- [ ] The final refactor checkpoint includes a clean validation run and recorded results.

### 17. Final Cleanup

- [ ] No known structural warnings remain unresolved.
- [ ] No major temporary architectural compromises remain untracked.
- [ ] No “we should move this later” bottlenecks remain in core files without being addressed.
- [ ] File naming and placement are consistent with project conventions.
- [ ] The remaining code reads as a maintained architecture, not a half-transition.

---

## Suggested Execution Order

Use this order unless a later item becomes blocked on a missing earlier extraction.

1. Finish native composition-root and tray cleanup.
2. Finish native controller/service boundary cleanup.
3. Finish frontend screen and view decomposition.
4. Finish frontend controller/state boundary cleanup.
5. Remove remaining duplication and legacy placements.
6. Reconcile the final structure against `ARCHITECTURE.md`.
7. Run final validation and close any remaining gaps.

---

## Tracking Rule

Only mark a checkbox complete when:

- the code has actually been changed to satisfy it
- the resulting structure is stable enough that the responsibility is unlikely to regress
- any relevant tests or validations for that slice have been run

If work is partial, leave the box unchecked.
