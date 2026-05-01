
---

## Mission

See [`README.md`](./README.md) for the full mission statement and product overview.

---

## Architecture Philosophy

ProjectLog uses a strict three-layer MVC split, adapted to the idioms of **Svelte on the frontend** and **Tauri + Rust on the native side**:

| Layer | Where it lives | Responsibility |
|---|---|---|
| **Model** | `src/models/` and `src-tauri/src/models/` | Shape of data — TypeScript/Rust domain types, validation, persistence abstractions, repository contracts |
| **View** | `src/views/` | Pure presentation — Svelte components that receive state and callbacks, with no business decisions |
| **Controller** | `src/controllers/` and `src-tauri/src/controllers/` | Business logic — orchestrates models, drives UI state, coordinates native commands, and enforces workflows |

Because ProjectLog is a desktop app with a frontend/backend split, MVC applies on **both sides**:

- The **frontend MVC** controls rendering, interaction flow, and view state.
- The **native MVC** controls persistence, file I/O, timesheet generation, tray behavior, and operating-system integration.

The boundary between them is explicit:

- **Frontend controllers** may call typed command clients in `src/services/bridge/`
- **Views** never call Tauri commands directly
- **Rust commands** are transport endpoints only; they delegate immediately to native controllers/services

This architecture is intentionally self-documenting. A developer should be able to infer *what belongs where* from folder names alone.

---

## Stack

| Concern | Choice | Notes |
|---|---|---|
| Desktop shell | Tauri v2 | Lightweight desktop app shell with native windowing, tray, autostart, updater support |
| Frontend UI | Svelte 5 | Reactive UI with clear component composition and low boilerplate |
| Frontend language | TypeScript | Strict typing for models, controller APIs, and bridge contracts |
| Build tool | Vite | Fast frontend build and local development |
| Native language | Rust | Reliable filesystem access, structured domain logic, and desktop integration |
| Testing | Playwright + Cargo tests | UI regression coverage plus Rust unit/integration tests |
| Packaging / updates | Tauri updater | App updates remain separate from domain logic |
| Persistence | Local files / native storage | Local-first storage for projects, logs, settings, and generated timesheet data |

---

## Architectural Goals

The architecture must optimize for the actual purpose of the product:

1. **Fast project switching** — selecting what you are working on should take seconds, from either surface.
2. **Reliable daily hour logging** — users must be able to reconstruct hours spent per project each day with confidence.
3. **Dual-surface consistency** — QuickPanel and the system tray operate on the same domain state; neither surface owns data the other cannot see.
4. **Local-first trust** — project names, comments, timestamps, and settings remain on the user's machine with no network dependency.
5. **Safe refactoring** — the codebase should support aggressive reorganization without blurring responsibilities.
6. **Clear expansion path** — reminders, timesheet rounding, export formats, tray flows, and reporting must fit the architecture cleanly.

---

## Two Surfaces, One Domain

ProjectLog presents two interaction surfaces over the same underlying domain:

### QuickPanel
A floating Tauri window that gives the user full access to all features. It contains:
- A project list with sort modes (Manual, A-Z, Recent) and drag-to-reorder
- A comment field that attaches context to the active session
- Add project and Quick track (ad-hoc) inputs
- Timesheet actions: Full timesheet, Yesterday + today, Open log file, Reset timesheet
- Settings: Always on top, Open on start, Opacity, Compact mode

### System Tray
A native Windows system tray menu that mirrors the core QuickPanel actions without requiring the window to be open:
- Project list for direct activation
- Set comment
- Add project / Quick project / Remove project
- Generate timesheet (with submenu for range), Open log file, Reset timesheet, Reset projects
- Open diagnostic log, Feedback, About, Exit

**Architectural implication:** neither surface owns the domain. Both call into the same native controllers through Tauri commands or tray event handlers. State is owned by the native layer and returned to whichever surface is active.

---

## Data Flow

Strict one-directional flow. No layer may import from a layer above it.

### Frontend flow

```
Svelte View
    ↓ user action
Frontend Controller
    ↓ calls
Bridge Service
    ↓ invokes
Tauri Command
    ↓ delegates
Native Controller
    ↓ reads/writes
Repository / Service
    ↓ side effects
Files / OS APIs / Window APIs / Tray / Updater
```

### State returning to UI

```
Repository / Native Service
    ↓ returns DTO / emits event
Native Controller
    ↓ through Tauri command or event
Bridge Service
    ↓ normalizes
Frontend Controller
    ↓ updates view model / store
Svelte View
```

### Tray flow (no frontend involved)

```
User clicks tray menu item
    ↓
Tray event handler (infrastructure/tray.rs)
    ↓ delegates
Native Controller
    ↓ reads/writes
Repository / Service
    ↓ emits Tauri window event (if QuickPanel is open)
Frontend Controller re-syncs store
```

**Hard rules:**

- Views never import from native bridge internals, repositories, or persistence code
- Views never call `invoke` directly
- Frontend controllers never import Svelte view components
- Bridge services never contain business rules
- Tauri commands never contain domain logic beyond transport mapping
- Native repositories never depend on window, tray, or UI concerns
- Tray handlers delegate to controllers immediately — they are not controllers themselves
- Shared types must live in dedicated model/type folders, not inside views or controllers

---

## Domain Boundaries

ProjectLog is organized around six stable business domains.

### 1. Project Domain

Responsible for:
- permanent projects
- ad-hoc / quick projects
- manual ordering
- sort modes: Manual, A-Z, Recent
- recent usage metadata for sort and display

### 2. Session / Log Domain

Responsible for:
- active project state (which project is currently being tracked)
- start/stop transitions and timestamp recording
- comment attachment to the active session block
- append-only log entries that preserve full history
- reconstructing daily work history from log data

### 3. Timesheet Domain

Responsible for:
- aggregating log entries into per-project daily hour totals
- producing a weekly table (Mon–Sun columns, project rows, comment sub-rows)
- range selection: full history, this week, yesterday + today
- rounding rules (e.g. round to 0.5h)
- preview generation for the Timesheet window
- export serialization to Excel

This domain is core to the product mission. The timesheet output is the primary deliverable users rely on for daily hour reporting.

### 4. Settings Domain

Responsible for:
- always-on-top window behavior
- open QuickPanel on system start
- QuickPanel opacity (0–100%)
- compact mode (reduced window height)
- window position and size persistence
- timesheet rounding rules

### 5. Desktop Shell Domain

Responsible for:
- system tray menu construction and state
- tray menu item enabling/disabling based on active session
- window visibility, focus, and placement
- autostart integration
- updater workflows
- desktop lifecycle events (minimize to tray, restore, close)

### 6. Diagnostics Domain

Responsible for:
- structured diagnostic log for troubleshooting
- app health inspection (storage paths, file integrity, version)
- debug-safe support output
- migration/format upgrade visibility

---

## Recommended Full Tree

```text
projectlog/
│
├── src/                                            # Frontend (Svelte + TypeScript)
│   │
│   ├── models/                                     # [MODEL] Data shapes, validation schemas, view models
│   │   ├── types/
│   │   │   ├── index.ts                            # Barrel export
│   │   │   ├── project.types.ts                    # Project, AdHocProject, SortMode, RecentMeta
│   │   │   ├── session.types.ts                    # ActiveSession, LogEntry, TransitionEvent
│   │   │   ├── timesheet.types.ts                  # DailySummary, ProjectHours, TimesheetRow,
│   │   │   │                                       # CommentRow, RangeSelection, RoundingRule
│   │   │   ├── settings.types.ts                   # AppSettings, OpacityValue, CompactMode
│   │   │   └── diagnostics.types.ts                # DiagnosticsReport, MigrationStatus
│   │   └── schemas/
│   │       ├── ProjectSchema.ts                    # Validates project name input
│   │       ├── LogEntrySchema.ts                   # Validates log entry payloads from native
│   │       └── SettingsSchema.ts                   # Validates settings DTO from native
│   │
│   ├── controllers/                                # [CONTROLLER] Business logic — no markup, no invoke calls
│   │   ├── projects/
│   │   │   ├── createProjectListController.ts      # Load, sort, reorder, select, deselect projects
│   │   │   └── createQuickEntryController.ts       # Quick-track (ad-hoc) input and submission
│   │   ├── sessions/
│   │   │   └── createSessionController.ts          # Start/stop tracking, attach comment, sync active state
│   │   ├── timesheets/
│   │   │   └── createTimesheetController.ts        # Range selection, rounding toggle, preview load, export
│   │   ├── settings/
│   │   │   └── createSettingsController.ts         # Load/save settings, opacity, compact mode, always-on-top
│   │   └── diagnostics/
│   │       └── createDiagnosticsController.ts      # Fetch and expose app health state and log path
│   │
│   ├── views/                                      # [VIEW] Pure presentation — props in, callbacks out
│   │   │                                           # No business logic. No invoke calls. No store imports.
│   │   ├── components/
│   │   │   ├── ui/                                 # Design-system primitives — zero domain knowledge
│   │   │   │   ├── Button.view.svelte
│   │   │   │   ├── Input.view.svelte
│   │   │   │   ├── Toggle.view.svelte
│   │   │   │   ├── Slider.view.svelte
│   │   │   │   ├── Spinner.view.svelte
│   │   │   │   └── index.ts                        # Barrel export
│   │   │   │
│   │   │   ├── projects/
│   │   │   │   ├── ProjectList.view.svelte         # Ordered list of projects; sort tabs at top
│   │   │   │   ├── ProjectRow.view.svelte          # Row: drag handle, name, active indicator, remove (x)
│   │   │   │   ├── SortTabs.view.svelte            # Manual / A-Z / Recent tab strip
│   │   │   │   └── AddProjectInput.view.svelte     # "Add project" input + Add button
│   │   │   │
│   │   │   ├── sessions/
│   │   │   │   ├── ActiveSessionHeader.view.svelte # "No active project" / active project name display
│   │   │   │   ├── CommentInput.view.svelte        # Comment field + Save / Clear buttons
│   │   │   │   └── QuickTrackInput.view.svelte     # "Quick project" input + Track button
│   │   │   │
│   │   │   ├── timesheets/
│   │   │   │   ├── TimesheetTable.view.svelte      # Weekly grid: project rows, comment sub-rows,
│   │   │   │   │                                   # Mon–Sun columns, Total column, Total footer row
│   │   │   │   ├── RangeSelector.view.svelte       # Full / Yesterday + today / custom range
│   │   │   │   ├── RoundingToggle.view.svelte      # "Round to 0.5h" toggle in timesheet footer
│   │   │   │   ├── GeneratedTimestamp.view.svelte  # "Generated at …, N seconds ago" + Update now
│   │   │   │   └── ExportButton.view.svelte        # Export to Excel button
│   │   │   │
│   │   │   ├── settings/
│   │   │   │   ├── SettingsPanel.view.svelte       # Settings section of QuickPanel
│   │   │   │   ├── OpacitySlider.view.svelte       # Opacity label + slider + percentage display
│   │   │   │   └── CompactModeToggle.view.svelte
│   │   │   │
│   │   │   └── diagnostics/
│   │   │       └── DiagnosticsPanel.view.svelte    # Storage paths, version, migration status
│   │   │
│   │   └── screens/
│   │       ├── QuickPanelScreen.svelte             # Primary floating window — composes all panel sections
│   │       ├── TimesheetScreen.svelte              # Dedicated timesheet preview window
│   │       └── DiagnosticsScreen.svelte            # Diagnostic log viewer window
│   │
│   ├── services/
│   │   └── bridge/                                 # Typed Tauri command clients — called by controllers only
│   │       │                                       # Each file wraps `invoke` for one domain.
│   │       │                                       # Views and stores never import from here.
│   │       ├── projectBridge.ts                    # add, remove, reorder, select, list
│   │       ├── sessionBridge.ts                    # startTracking, stopTracking, setComment
│   │       ├── timesheetBridge.ts                  # generate, getPreview, exportToExcel, reset
│   │       ├── settingsBridge.ts                   # load, save, applyOpacity, setAlwaysOnTop
│   │       └── diagnosticsBridge.ts                # getReport, openLogFile
│   │
│   ├── stores/                                     # Svelte stores — written by controllers, read by views
│   │   ├── projectStore.ts                         # Ordered project list, active project, sort mode
│   │   ├── sessionStore.ts                         # Active session state, current comment text
│   │   ├── timesheetStore.ts                       # Preview rows, selected range, rounding state
│   │   └── settingsStore.ts                        # App settings, opacity, compact mode flag
│   │
│   └── utils/                                      # Pure stateless helpers — no imports from src/
│       ├── formatters.ts                           # Duration display, date formatting, relative time
│       └── sanitize.ts                             # Project name trimming and validation helpers
│
│
├── src-tauri/src/                                  # Native (Rust)
│   │
│   ├── commands/                                   # Tauri transport endpoints — delegate immediately, no logic
│   │   │                                           # A command that grows domain logic is a bug.
│   │   ├── project_commands.rs
│   │   ├── session_commands.rs
│   │   ├── timesheet_commands.rs
│   │   ├── settings_commands.rs
│   │   └── diagnostics_commands.rs
│   │
│   ├── controllers/                                # [CONTROLLER] Domain rules and workflow orchestration
│   │   ├── project_controller.rs                  # Add, remove, reorder, select, list projects;
│   │   │                                           # enforce name uniqueness; update recent metadata
│   │   ├── session_controller.rs                  # Start/stop tracking; write log entry on transition;
│   │   │                                           # attach comment to active session block
│   │   ├── timesheet_controller.rs                # Delegate to timesheet_service; apply rounding;
│   │   │                                           # coordinate export; enforce no-mutation-of-source rule
│   │   ├── settings_controller.rs                 # Load, merge, validate, and persist settings;
│   │   │                                           # notify shell of opacity/always-on-top changes
│   │   ├── shell_controller.rs                    # Window show/hide, position restore, tray sync,
│   │   │                                           # compact mode application, autostart toggle
│   │   └── diagnostics_controller.rs              # Collect health report; open diagnostic log in OS viewer
│   │
│   ├── models/                                     # [MODEL] Domain types, DTOs, and repository traits
│   │   ├── domain/
│   │   │   ├── project.rs                         # Project, AdHocProject, SortMode, RecentMeta
│   │   │   ├── log_entry.rs                       # LogEntry, TransitionEvent, CommentBlock
│   │   │   ├── timesheet.rs                       # DailySummary, ProjectHours, TimesheetRow,
│   │   │   │                                       # CommentRow, RangeSelection, RoundingRule
│   │   │   └── settings.rs                        # AppSettings, OpacityValue, WindowBounds
│   │   ├── dto/
│   │   │   ├── project_state_dto.rs               # Serializable project list + active project for frontend
│   │   │   ├── timesheet_dto.rs                   # Weekly table rows for frontend preview
│   │   │   └── settings_dto.rs                    # Serializable settings snapshot for frontend
│   │   └── repository_traits/
│   │       ├── project_repository.rs              # trait ProjectRepository — list, add, remove, reorder
│   │       ├── log_repository.rs                  # trait LogRepository — append, list_for_day, list_range
│   │       └── settings_repository.rs             # trait SettingsRepository — load, save
│   │
│   ├── repositories/                               # Concrete persistence — controllers never know the format
│   │   ├── file_project_repository.rs             # Implements ProjectRepository over local JSON file
│   │   ├── file_log_repository.rs                 # Implements LogRepository; append-only log format
│   │   └── file_settings_repository.rs            # Implements SettingsRepository over local config file
│   │
│   ├── services/                                   # Domain services — called by controllers, not commands
│   │   ├── timesheet_service.rs                   # Aggregates log entries → per-project daily hour totals;
│   │   │                                           # applies rounding; builds preview rows and comment sub-rows;
│   │   │                                           # produces export-ready data; never mutates source log
│   │   ├── migration_service.rs                   # Startup detection of legacy file formats;
│   │   │                                           # upgrades format in place; records migration in diagnostic log
│   │   └── export_service.rs                      # Serializes timesheet DTO to Excel (.xlsx) format
│   │
│   ├── infrastructure/                             # OS-level glue — no domain logic lives here
│   │   ├── tray.rs                                # Tray menu construction, item state, event dispatch;
│   │   │                                           # maps tray events to controller calls;
│   │   │                                           # syncs enabled/disabled state from active session
│   │   ├── window.rs                              # Window creation, show/hide, bounds save/restore,
│   │   │                                           # always-on-top, opacity application
│   │   └── autostart.rs                           # OS autostart registry integration
│   │
│   └── state.rs                                    # AppState — wires all controllers into Tauri manage();
│                                                   # single shared instance across commands and tray handlers
│
│
├── tests/
│   ├── e2e/                                        # Playwright end-to-end tests against the real app
│   │   ├── quick_track.spec.ts                     # Critical: fast project switching in QuickPanel
│   │   ├── comment_attach.spec.ts                  # Critical: comment saved against correct session
│   │   ├── timesheet_preview.spec.ts               # Critical: daily hour totals are accurate
│   │   ├── timesheet_rounding.spec.ts              # Critical: rounding rules applied correctly
│   │   ├── tray_project_select.spec.ts             # Critical: tray project selection updates QuickPanel
│   │   ├── settings_persist.spec.ts                # Window bounds, opacity, and compact mode survive restart
│   │   └── export_excel.spec.ts                   # Export produces valid .xlsx without corrupting log
│   │
│   └── rust/                                       # Cargo integration tests (complement to in-module unit tests)
│       ├── timesheet_aggregation.rs               # Multi-day log → correct daily totals, comment grouping
│       ├── timesheet_rounding.rs                  # Edge cases: sub-interval, overnight, zero entries
│       └── migration.rs                           # Legacy format detected, upgraded, and re-readable
│
│
├── scripts/
│   ├── save-wip.ps1
│   ├── push-wip.ps1
│   ├── release.mjs
│   ├── sync-version.mjs
│   └── push-release.ps1
│
├── docs/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── svelte.config.js
├── playwright.config.ts
└── README.md
```

---

## Why This Tree Fits ProjectLog

The structure is organized around the actual domains of the product rather than the current files:

- **Projects** — what the user works on; ordered, sortable, permanent or ad-hoc
- **Sessions / log entries** — when they worked on it; start/stop timestamps with optional comments
- **Timesheets** — the primary deliverable: per-project daily hour totals derived from log entries
- **Settings** — startup behavior, window behavior, opacity, compact mode, rounding rules
- **Desktop shell** — tray, window lifecycle, and OS integration, isolated from domain logic
- **Diagnostics** — operational transparency for debugging and support

This separation ensures that "generate daily project hour insight" is treated as a first-class domain concern rather than a utility feature bolted onto the UI.

---

## Key Conventions

### File Naming

| Artefact | Convention | Example |
|---|---|---|
| Svelte view component | `Name.view.svelte` | `ProjectRow.view.svelte` |
| Screen | `NameScreen.svelte` | `TimesheetScreen.svelte` |
| Frontend controller | `createXController.ts` | `createTimesheetController.ts` |
| Bridge service | `xBridge.ts` | `timesheetBridge.ts` |
| Store | `xStore.ts` | `projectStore.ts` |
| Rust controller | `x_controller.rs` | `timesheet_controller.rs` |
| Repository trait | `x_repository.rs` | `log_repository.rs` |
| Concrete repository | `file_x_repository.rs` | `file_log_repository.rs` |
| Domain model | `x.rs` | `log_entry.rs` |
| DTO | `x_dto.rs` | `timesheet_dto.rs` |
| Tauri commands | `x_commands.rs` | `settings_commands.rs` |
| Validation schema | `NameSchema.ts` | `SettingsSchema.ts` |

### Dependency Rule

Each layer may only import from layers *below* it. Violations are treated as bugs.

#### Frontend

```text
┌────────────────────────────────────┐
│ View (screens, components)         │ ← imports controllers, view models, ui primitives
├────────────────────────────────────┤
│ Controllers                        │ ← imports bridge services, stores, models
├────────────────────────────────────┤
│ Bridge services / stores           │ ← imports models and transport helpers
├────────────────────────────────────┤
│ Models / schemas / mappers         │ ← imports utility-only helpers
└────────────────────────────────────┘
```

#### Native

```text
┌────────────────────────────────────┐
│ Tauri commands / tray handlers     │ ← imports controllers only
├────────────────────────────────────┤
│ Controllers                        │ ← imports repositories, services, state, models
├────────────────────────────────────┤
│ Repositories / services            │ ← imports models, infrastructure, utils
├────────────────────────────────────┤
│ Models / DTOs / traits             │ ← no UI concerns, no OS concerns
└────────────────────────────────────┘
```

### Frontend Controller Pattern

Frontend controllers are the only layer screen components interact with. They own async state, call bridge services, normalize data, update stores, and expose a clean action surface. They do not render markup.

```ts
// src/controllers/projects/createProjectListController.ts
export function createProjectListController(deps: {
  projectBridge: ProjectBridge;
  projectStore: ProjectStore;
}) {
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function selectProject(name: string) {
    loading = true;
    error = null;
    try {
      const nextState = await deps.projectBridge.selectProject(name);
      deps.projectStore.apply(nextState);
    } catch {
      error = "Could not select project.";
    } finally {
      loading = false;
    }
  }

  return { selectProject, get loading() { return loading; }, get error() { return error; } };
}
```

### Native Controller Pattern

Native controllers own application rules. Tauri commands and tray handlers should forward to them immediately — neither is a substitute for a controller.

```rust
// src-tauri/src/controllers/session_controller.rs
pub struct SessionController<L: LogRepository, P: ProjectRepository> {
    log_repo: L,
    project_repo: P,
}

impl<L: LogRepository, P: ProjectRepository> SessionController<L, P> {
    pub fn start_tracking(&self, project: &str) -> Result<ProjectStateDto, String> {
        let entry = LogEntry::start(project, Utc::now());
        self.log_repo.append(entry)?;
        self.project_repo.set_active(project)?;
        self.project_repo.record_recent_usage(project)?;
        self.project_repo.get_state()
    }
}
```

### Repository Pattern

Repositories abstract persistence. Controllers should not care whether project data lives in flat files today or SQLite tomorrow.

```rust
// src-tauri/src/models/repository_traits/log_repository.rs
pub trait LogRepository {
    fn append(&self, entry: LogEntry) -> Result<(), String>;
    fn list_for_day(&self, day: NaiveDate) -> Result<Vec<LogEntry>, String>;
    fn list_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<LogEntry>, String>;
}
```

### Command Boundary Pattern

Tauri commands are transport adapters, not business modules.

```rust
// src-tauri/src/commands/session_commands.rs
#[tauri::command]
pub fn start_tracking(project: String, state: State<AppState>) -> Result<ProjectStateDto, String> {
    state.session_controller.start_tracking(&project)
}
```

If a command grows real business logic, that logic belongs in a controller.

### Tray Handler Pattern

Tray handlers are event adapters, not business modules. They mirror the command boundary pattern on the native side.

```rust
// src-tauri/src/infrastructure/tray.rs
SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
    project if projects.contains(project) => {
        app.state::<AppState>().session_controller.start_tracking(project).ok();
        sync_quickpanel_if_open(&app);
    }
    "open_quickpanel" => { shell_controller.show_quickpanel(); }
    _ => {}
}
```

### View Pattern

Views receive already-prepared state and emit intent callbacks. A view may know *what to show*, but never *what the business rules are*.

```svelte
<!-- src/views/components/projects/ProjectRow.view.svelte -->
<script lang="ts">
  let {
    name,
    isActive,
    onSelect,
    onRemove,
  }: {
    name: string;
    isActive: boolean;
    onSelect: (name: string) => void;
    onRemove: (name: string) => void;
  } = $props();
</script>
```

### Self-Documenting Folder Rule

A folder should answer one of these questions immediately:

- **Is this domain shape?** → `models/`
- **Is this business behavior?** → `controllers/`
- **Is this rendering?** → `views/`
- **Is this transport or side effects?** → `services/bridge/` or `commands/`
- **Is this persistence detail?** → `repositories/`
- **Is this operating-system glue?** → `infrastructure/`

If a file cannot be placed confidently, that usually indicates the responsibility is still unclear.

---

## Testing Philosophy

The MVC split makes each layer independently testable with small, focused mocks.

| Layer | What to test | Mock boundary |
|---|---|---|
| Frontend `models/schemas` | Payload parsing and invalid-state rejection | None needed |
| Frontend controllers | Async state, command sequencing, error handling | Mock bridge services |
| Frontend views | Render behavior, callbacks, keyboard and input flows | Mock controllers / props |
| Bridge services | Command/event normalization | Mock `invoke` / event APIs |
| Native controllers | Domain workflows and rule enforcement | Mock repositories and services |
| Native repositories | File/storage behavior and data mapping | Mock filesystem / infrastructure |
| `timesheet_service` | Hour aggregation, rounding, comment grouping, range logic | Mock repositories or clocks |
| `migration_service` | Legacy format detection, upgrade correctness | Mock filesystem |
| End-to-end | Real desktop behavior across both surfaces | Playwright |

**Critical regression paths for ProjectLog:**

1. Selecting a project in QuickPanel starts a session and updates tray state
2. Selecting a project in the tray starts a session and updates QuickPanel if open
3. Saving a comment attaches it to the correct session block in the log
4. Timesheet preview shows correct per-project daily hour totals
5. Rounding toggle changes displayed totals without mutating source log
6. Exporting to Excel produces a valid file matching the preview
7. Settings (opacity, always-on-top, compact mode) survive app restart
8. Window bounds are restored correctly on reopen

---

## Persistence Philosophy

ProjectLog is a local-first tool. Persistence rules should be explicit:

1. User project data, comments, timestamps, and settings are stored locally only — no cloud, no auth.
2. The log is append-only. Timesheet generation reads from it; nothing writes to it except session transitions.
3. Persistence format is an implementation detail hidden behind repository traits.
4. The app must tolerate missing, partial, or legacy files through startup migration services.
5. Timesheet generation must be fully reproducible from persisted log data at any time.
6. Export logic must never mutate source log history.

---

## Refactor Guidance

This architecture is intended to guide a heavy refactor. During that refactor:

- do **not** move business logic into Svelte views
- do **not** let Tauri commands become controller substitutes
- do **not** let tray handlers become controller substitutes
- do **not** hide domain rules inside utility files
- do **not** let desktop-shell code leak into project/session/timesheet domains
- do **not** optimize for current file layout over long-term clarity

The goal is not to mirror the current project structure. The goal is a codebase where the architecture itself explains how ProjectLog helps users track work daily and produce trustworthy hour summaries.

---

## Summary

ProjectLog is refactored into a dual-layer MVC system:

- **Frontend MVC** for Svelte rendering and interaction in the QuickPanel and Timesheet windows
- **Native MVC** for Rust domain logic, persistence, tray, and OS integration
- **Thin transport boundary** between them via typed bridge services and Tauri commands
- **Two surfaces (QuickPanel, tray) over one domain** — neither surface owns state

That structure best serves the product mission: a fast, private, local-only desktop tool for logging what was worked on each day, how long it took, and turning that activity into reliable project-hour insight.
