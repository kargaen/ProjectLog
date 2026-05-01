# Workflows Template

Use this file as a portable checklist when copying the development and release setup into another app.
Replace project-specific names, commands, packaging details, and UI behavior to fit the new app.

## 1. Local Developer Entry Points

- Dev server flow:
  Provide a single command for local frontend or app-shell development, such as `npm run dev`.
- Production build flow:
  Provide a single command for production frontend output, such as `npm run build`.
- Desktop app entry flow:
  If the app uses a shell like Tauri or Electron, provide a standard command such as `npm run tauri` or `npm run electron`.
- Preview flow:
  Optionally provide a `preview` command for checking the production frontend locally.

## 2. Editor Task Shortcuts

- Save WIP shortcut:
  Bind a key to a task that creates a quick checkpoint commit.
- Release shortcut:
  Bind a key to a task that starts the interactive release flow.
- Run tests shortcut:
  Bind a key to a task that runs the full local test chain.
- Shared terminal presentation:
  Configure tasks to reveal output clearly, reuse one terminal panel, and clear previous output.
- Portability rule:
  Keep task definitions in workspace files, but assume keybindings may need to be copied into user or profile settings on each machine.

## 3. WIP Commit Flow

- Timestamped checkpoint commit:
  Add a script that stages all changes and creates a timestamped WIP commit only if the working tree is dirty.
- Clean-tree skip behavior:
  If there are no changes, exit cleanly and tell the user no commit was created.
- Optional WIP push flow:
  Add a companion script that pushes the current branch and creates upstream tracking automatically if missing.

## 4. Single Source Of Truth Versioning

- Canonical version source:
  Choose one file as the version source of truth, usually `package.json` or a dedicated app manifest.
- Version sync script:
  Add a script that propagates the chosen version into every other file that must match it.
- Version validation:
  Reject invalid versions early using a semver-like validation rule.
- Post-bump hook:
  Optionally add an npm or package-manager hook so version synchronization runs automatically after version changes.

## 5. Release Notes As Part Of Development

- Rolling release log:
  Maintain a file where user-facing changes are written as they are implemented. New user-facing features and noteworthy fixes should be written to `RELEASE_LOG.md` as they are implemented.
- Release message source:
  Use that same file as the release commit message source instead of typing a message at release time.
- Reset after release:
  After a successful release prep, restore the release-log file to a reusable template.
- Team rule:
  Document that notable user-facing work should be added to the release log during implementation, not at the end.

## 6. Interactive Release Flow

- Interactive version prompt:
  Ask for the new version during release if it is not passed explicitly.
- Confirmation step:
  Require the version to be entered twice or otherwise confirmed before proceeding.
- Tag collision protection:
  Refuse to release if the target git tag already exists.
- File backup and restore:
  Back up the versioned files and restore them automatically if release setup fails before the commit is created.
- Automated release commit:
  Sync version files, reset the release log template, stage all changes, and create one intentional release commit.
- Tag creation:
  Create a `v<version>` tag as part of the same release flow.
- Push branch and tag:
  Push the current branch first, then push the tag.
- Upstream detection:
  If the branch has no upstream, create it automatically during push.

## 7. Local Test Flow

- Fast backend/core tests:
  Provide one command for backend or core tests that can run without the full app shell.
- UI/browser tests:
  Provide one command for UI tests.
- Combined local verification task:
  Chain the core tests and UI tests in one editor task or script so a developer can run the full validation path quickly.
- Fail-fast sequencing:
  Stop before UI tests if backend or core tests fail.

## 8. UI Automation Flow

- Browser test runner:
  Use a framework such as Playwright for UI-level regression coverage.
- Auto-start dev server:
  Configure the UI test runner to boot the local dev server automatically and reuse it when already running.
- Desktop-shell mocking:
  If the app runs in a desktop shell, mock the shell APIs so UI behavior can be tested without launching the native app every time.
- Focused regression scenarios:
  Cover key interaction flows, state persistence, dedicated windows, and recent bug-prone UI paths.
- Retry diagnostics:
  Capture traces, screenshots, or equivalent artifacts on retries or failures.

## 9. CI Validation Flow

- Version consistency gate:
  In CI, verify that all files expected to share the version actually match.
- Tag-to-version gate:
  If releasing from tags, verify that the tag matches the app version exactly.
- Frontend build gate:
  Build the frontend before release packaging.
- Core test gate:
  Run the backend or core test suite before packaging.
- Optional UI test gate:
  If practical in CI, also run UI tests or a smaller smoke suite.

## 10. Release CI / CD Flow

- Trigger on release tags:
  Run the release workflow automatically on tags like `v*`. The release workflow uses `RELEASE_LOG.md` as the release commit message, then clears the file back to its template after a successful release.
- Manual dispatch option:
  Also support manual triggering with an optional version input.
- Shared version sync in CI:
  Reuse the same local version sync script in the workflow so local and CI behavior stay aligned.
- Platform packaging:
  Build the installer, app bundle, or archive for the target platform.
- Release asset publishing:
  Upload artifacts directly to the source-control release page.
- Prerelease detection:
  Automatically mark prereleases based on the version format when relevant.
- Build caching:
  Cache dependencies and compiled artifacts to speed up repeated release builds.

## 11. Auto-Update Flow

- Updater artifact generation:
  Configure packaging to create update metadata and signed artifacts if the app supports self-updates.
- Stable update endpoint:
  Point the app updater at a predictable release metadata URL.
- Release compatibility:
  Ensure CI publishes the exact files the in-app updater expects.

## 12. Lightweight Website / Landing Page Flow

- Auto-generated landing page:
  Optionally generate a very small static site directly from workflow steps instead of maintaining a separate site project.
- Version-stamped download page:
  Show the current app version and link to the latest release.
- Privacy or policy page:
  Generate simple support pages alongside the landing page.
- Deploy from CI:
  Publish the site automatically on main branch pushes, successful releases, or both.

## 13. Architecture Support Flow

- Shared app services:
  Keep versioning, release prep, testing, and distribution logic in shared scripts or services rather than duplicating steps in many places.
- Explicit cross-window coordination:
  In desktop apps with multiple windows or webviews, let views raise intents and let shared application logic decide how windows are refreshed, reopened, or synchronized.
- Deterministic fallback behavior:
  For tricky multi-window refresh cases, prefer a simple “close and recreate with fresh state” flow over fragile incremental updates.

## 14. Practical Files To Recreate In Another Project

- `package.json` scripts section
- `.vscode/tasks.json`
- `.vscode/keybindings.json`
- `scripts/save-wip.ps1`
- `scripts/push-wip.ps1`
- `scripts/release.mjs`
- `scripts/sync-version.mjs`
- optional `scripts/push-release.ps1`
- `playwright.config.*`
- `tests/ui/*`
- desktop-shell mocks for UI tests
- `RELEASE_LOG.md`
- CI workflow files for release and site deployment

## 15. Migration Checklist For A New App

- Decide the version source of truth.
- List every file that must share the version.
- Create a version sync script first.
- Add a rolling release-log file and document how it is used.
- Add WIP commit and push scripts.
- Add editor tasks and optional shortcuts.
- Add core tests and one combined local test task.
- Add UI automation with shell mocks if this is a desktop app.
- Add release CI that validates versions before packaging.
- Add updater metadata generation if the app supports auto-update.
- Add a minimal landing-page workflow only if the app benefits from it.
