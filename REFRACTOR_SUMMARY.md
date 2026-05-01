# Refactor Summary

## Outcome

The refactor moved ProjectLog meaningfully closer to the intended MVC architecture without forcing a risky full rewrite.

The codebase now has clearer architectural boundaries, stronger documentation, better workflow support, and a more useful automated test baseline. The app remains functional, and the current state is good enough to resume from later without having to reconstruct why changes were made.

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

## What worked well

- Using architecture documentation as the source of truth before pushing further refactor changes.
- Moving native logic into controllers without attempting to reorganize everything at once.
- Keeping browser tests focused on user-visible workflows instead of overfitting to implementation detail.
- Using Rust tests for timesheet correctness and Playwright for interaction confidence.
- Using richer mock data to stabilize UI tests.

## What remains unfinished

### 1. Frontend MVC is still incomplete

The frontend is not yet aligned with the target architecture described in `ARCHITECTURE.md`.

Most notably:
- `src/App.svelte` is still too monolithic
- passive views, frontend controllers, and bridge/service layers are not yet cleanly separated
- some domain responsibilities are still too close to the UI surface

This is likely the next major refactor area.

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
- Playwright UI tests: `8 passed`

This means the current checkpoint is stable enough to continue from later.

## Recommended next steps

When work resumes, the most sensible order is:

1. Refactor the frontend toward the documented MVC target.
   - Break up `src/App.svelte`
   - Introduce frontend controller/service boundaries
   - Keep views passive

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
- test coverage is stronger and more realistic
- future work is easier to reason about

The next session should treat this as a guided continuation, not as a recovery task.
