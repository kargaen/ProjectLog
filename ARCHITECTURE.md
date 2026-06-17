---
## Mission

See [`README.md`](./README.md) for the full mission statement and product overview.

---

## Five Design Decisions That Keep This Codebase Stable

These five decisions are the load-bearing walls of the architecture. Every naming convention, folder rule, and layer boundary below exists to enforce them. When a bug cascades, it is almost always because one of these was violated.

**1. The import graph flows in one direction only.**
Views import from controllers. Controllers import from bridge services and stores. Bridge services import from models. Nothing imports from a layer above it. This means you can always locate a bug by asking which layer owns the broken invariant — there is no "could be anywhere" debugging.

**2. Repositories own every storage decision.**
Nothing above the repository layer knows whether data is in a flat file, JSON, or a different format. Controllers never call filesystem APIs or `invoke` storage commands directly. Swapping a storage backend or format requires changing exactly one file.

**3. Stores are write-through, not write-ahead.**
Controllers write to a Svelte store only after the native side effect (repository write via Tauri command) succeeds. The store is always a mirror of committed state, never an optimistic prediction. Transient UI state (loading flags, draft values, open dialogs) lives in the controller's local `$state`, not in shared stores.

**4. The component decomposition convention enforces zero logic in markup.**
Every non-trivial component splits into three files: `Name.view.svelte` for markup only, `Name.styles.ts` for all StyleSheet-equivalent style objects, and `Name.hooks.ts` for any formatting, local state, or derived values. The `.view.svelte` file may only contain JSX-equivalent markup and prop destructuring. No conditionals, no format calls, no event logic beyond passing callbacks through.

**5. Two surfaces share one domain. Neither owns state.**
The QuickPanel window and the system tray are both surfaces over the same native domain state. Neither surface caches, derives, or manages its own version of the truth. Both call into the same native controllers. When either surface acts, the result is a fresh state snapshot returned from the native layer, which both surfaces then reflect.

---

## Architecture Philosophy

ProjectLog uses a strict MVC split adapted to **Svelte 5 on the frontend** and **Tauri + Rust on the native side**:

| Layer          | Where it lives                                      | Responsibility                                                                                           |
| -------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Model**      | `src/models/` and `src-tauri/src/models/`           | Shape of data — TypeScript/Rust domain types, validation schemas, persistence abstractions, repository contracts |
| **View**       | `src/views/`                                        | Pure presentation — Svelte components that receive state and emit callbacks, with no business decisions  |
| **Controller** | `src/controllers/` and `src-tauri/src/controllers/` | Business logic — orchestrates models, drives UI state, coordinates native commands, enforces workflows   |

Because ProjectLog is a desktop app with a frontend/native split, MVC applies on **both sides**:

- The **frontend MVC** controls rendering, interaction flow, and view state.
- The **native MVC** controls persistence, file I/O, timesheet generation, tray behavior, and OS integration.

The boundary between them is a thin, typed transport layer:

- **Frontend controllers** call typed command clients in `src/services/bridge/`
- **Views** never call Tauri commands directly
- **Rust commands** are transport endpoints only — they delegate immediately to native controllers

---

## Stack

| Concern             | Choice                   | Notes                                                                                 |
| ------------------- | ------------------------ | ------------------------------------------------------------------------------------- |
| Desktop shell       | Tauri v2                 | Lightweight desktop shell with native windowing, tray, autostart, updater support    |
| Frontend UI         | Svelte 5                 | Reactive UI with rune-based reactivity and low boilerplate                            |
| Frontend language   | TypeScript               | Strict typing for models, controller APIs, and bridge contracts                      |
| Build tool          | Vite                     | Fast frontend build and local development                                             |
| Native language     | Rust                     | Reliable filesystem access, structured domain logic, and desktop integration          |
| Testing             | Playwright + Cargo tests | UI regression coverage plus Rust unit/integration tests                               |
| Packaging/updates   | Tauri updater            | App updates remain separate from domain logic                                         |
| Persistence         | Local files only         | Local-first storage for projects, logs, settings, and generated timesheet data        |

---

## Architectural Goals

The architecture must serve the actual purpose of the product:

1. **Fast project switching** — selecting what you are working on takes seconds, from either surface.
2. **Reliable daily hour logging** — users must be able to reconstruct hours spent per project each day with confidence.
3. **Dual-surface consistency** — QuickPanel and the tray operate on the same domain state; neither surface sees data the other cannot.
4. **Local-first trust** — project names, comments, timestamps, and settings remain on the user's machine with no network dependency.
5. **Safe refactoring** — the codebase must support aggressive reorganization without blurring responsibilities.
6. **Clear expansion path** — reminders, timesheet rounding, export formats, tray flows, and reporting must fit the architecture cleanly.

---

## Two Surfaces, One Domain

ProjectLog exposes two interaction surfaces over the same underlying native domain.

### QuickPanel

A floating Tauri window with full access to all features:

- Project list with sort modes (Manual, A-Z, Recent) and drag-to-reorder
- Comment field that attaches context to the active session
- Add project and Quick track (ad-hoc) inputs
- Timesheet actions: Full timesheet, Yesterday + today, Open log file, Reset timesheet
- Settings: Always on top, Open on start, Opacity, Compact mode

### System Tray

A native system tray menu that mirrors core QuickPanel actions without requiring the window to be open:

- Project list for direct activation
- Set comment
- Add / Quick add / Remove project
- Generate timesheet (range submenu), Open log file, Reset timesheet, Reset projects
- Open diagnostic log, Feedback, About, Exit

**Architectural implication:** neither surface owns the domain. Both call into the same native controllers through Tauri commands or tray event handlers. State is owned by the native layer and returned to whichever surface is active.

---

## Data Flow

Strict one-directional flow. No layer may import from a layer above it.

### Frontend flow

```
Svelte View
    ↓  user action (callback)
Frontend Controller
    ↓  calls
Bridge Service
    ↓  invoke()
Tauri Command
    ↓  delegates immediately
Native Controller
    ↓  reads / writes
Repository / Service
    ↓  side effects
Files / OS APIs / Window / Tray / Updater
```

### State returning to UI

```
Repository / Native Service
    ↓  returns DTO
Native Controller
    ↓  through Tauri command return value or event
Bridge Service
    ↓  deserializes and returns typed result
Frontend Controller
    ↓  writes to Svelte store (only on success)
Svelte View
    ↓  reactive derivation from store
```

### Ownership chain

```
Repository (reads/writes data)
    → Controller (orchestrates, enforces rules)
    → Store (broadcasts committed state)
    → View (renders)
```

### Tray flow (no frontend involved)

```
User clicks tray item
    ↓
Tray event handler (infrastructure/tray.rs)
    ↓  delegates immediately
Native Controller
    ↓  reads / writes
Repository / Service
    ↓  optionally emits Tauri window event if QuickPanel is open
Frontend Controller re-syncs store
```

### Hard rules

- Views never import from bridge services, repositories, or stores
- Views never call `invoke` directly
- Frontend controllers never import Svelte view components
- Bridge services never contain business rules
- Tauri commands never contain domain logic beyond transport mapping
- Native repositories never depend on window, tray, or UI concerns
- Tray handlers delegate to controllers immediately — they are not controllers themselves
- Stores are only written to after the native side effect succeeds
- Transient state (loading, draft values, dialog open/closed) lives in controller `$state`, not in stores
- Shared types live in `models/types/`, not inside views or controllers

---

## Domain Boundaries

ProjectLog is organized around six stable business domains.

### 1. Project Domain

- Permanent projects and ad-hoc / quick projects
- Manual ordering and drag-to-reorder
- Sort modes: Manual, A-Z, Recent
- Recent usage metadata for sort and display
- Name uniqueness enforcement

### 2. Session / Log Domain

- Active project state (which project is currently being tracked)
- Start/stop transitions and timestamp recording
- Comment attachment to the active session block
- Append-only log entries that preserve full history
- Reconstructing daily work history from log data

### 3. Timesheet Domain

- Aggregating log entries into per-project daily hour totals
- Producing a weekly table (Mon–Sun columns, project rows, comment sub-rows)
- Range selection: full history, this week, yesterday + today
- Rounding rules (e.g. round to 0.5h)
- Preview generation for the Timesheet window
- Export serialization to Excel

This domain is core to the product mission. The timesheet output is the primary deliverable users rely on for daily hour reporting.

### 4. Settings Domain

- Always-on-top window behavior
- Open QuickPanel on system start
- QuickPanel opacity (0–100%)
- Compact mode (reduced window height)
- Window position and size persistence
- Timesheet rounding rules

### 5. Desktop Shell Domain

- System tray menu construction and state
- Tray menu item enabling/disabling based on active session
- Window visibility, focus, and placement
- Autostart integration
- Updater workflows
- Desktop lifecycle events (minimize to tray, restore, close)

### 6. Diagnostics Domain

- Structured diagnostic log for troubleshooting
- App health inspection (storage paths, file integrity, version)
- Debug-safe support output
- Migration and format upgrade visibility

---

## Full Directory Tree

```text
projectlog/
│
├── src/                                            # Frontend (Svelte 5 + TypeScript)
│   │
│   ├── models/                                     # [MODEL] Data shapes and validation
│   │   ├── types/
│   │   │   ├── index.ts                            # Barrel export — only entry point for types
│   │   │   ├── project.types.ts                    # Project, AdHocProject, SortMode, RecentMeta
│   │   │   ├── session.types.ts                    # ActiveSession, LogEntry, TransitionEvent
│   │   │   ├── timesheet.types.ts                  # DailySummary, TimesheetRow, CommentRow,
│   │   │   │                                       # RangeSelection, RoundingRule
│   │   │   ├── settings.types.ts                   # AppSettings, OpacityValue, CompactMode
│   │   │   └── diagnostics.types.ts                # DiagnosticsReport, MigrationStatus
│   │   └── schemas/
│   │       ├── ProjectSchema.ts                    # Validates project name input at UI boundary
│   │       ├── LogEntrySchema.ts                   # Validates log entry payloads from native
│   │       └── SettingsSchema.ts                   # Validates settings DTO from native
│   │
│   ├── controllers/                                # [CONTROLLER] Business logic
│   │   │                                           # No markup. No invoke calls. No store imports
│   │   │                                           # from other domains.
│   │   ├── projects/
│   │   │   ├── createProjectListController.ts      # Load, sort, reorder, select, deselect projects
│   │   │   └── createQuickEntryController.ts       # Quick-track (ad-hoc) input and submission
│   │   ├── sessions/
│   │   │   └── createSessionController.ts          # Start/stop tracking, attach comment, sync active state
│   │   ├── timesheets/
│   │   │   └── createTimesheetController.ts        # Range selection, rounding toggle, preview load, export
│   │   ├── settings/
│   │   │   └── createSettingsController.ts         # Load/save settings, opacity, compact, always-on-top
│   │   └── diagnostics/
│   │       └── createDiagnosticsController.ts      # Fetch and expose app health state and log path
│   │
│   ├── views/                                      # [VIEW] Pure presentation
│   │   │                                           # Props in. Callbacks out. No business logic.
│   │   │                                           # No invoke calls. No store reads.
│   │   ├── components/
│   │   │   ├── ui/                                 # Design-system primitives — zero domain knowledge
│   │   │   │   ├── Button.view.svelte
│   │   │   │   ├── Input.view.svelte
│   │   │   │   ├── Toggle.view.svelte
│   │   │   │   ├── Slider.view.svelte
│   │   │   │   ├── Spinner.view.svelte
│   │   │   │   └── index.ts
│   │   │   │
│   │   │   ├── projects/
│   │   │   │   ├── ProjectList/
│   │   │   │   │   ├── ProjectList.view.svelte     # Ordered list + sort tabs
│   │   │   │   │   ├── ProjectList.styles.ts       # All style objects for this component
│   │   │   │   │   ├── ProjectList.hooks.ts        # Any formatting or derived values
│   │   │   │   │   └── index.ts
│   │   │   │   ├── ProjectRow/
│   │   │   │   │   ├── ProjectRow.view.svelte      # Drag handle, name, active indicator, remove
│   │   │   │   │   ├── ProjectRow.styles.ts
│   │   │   │   │   └── index.ts
│   │   │   │   ├── SortTabs.view.svelte            # Manual / A-Z / Recent tab strip
│   │   │   │   └── AddProjectInput.view.svelte     # "Add project" input + Add button
│   │   │   │
│   │   │   ├── sessions/
│   │   │   │   ├── ActiveSessionHeader.view.svelte # Active project name or "No active project"
│   │   │   │   ├── CommentInput.view.svelte        # Comment field + Save / Clear
│   │   │   │   └── QuickTrackInput.view.svelte     # Quick project input + Track button
│   │   │   │
│   │   │   ├── timesheets/
│   │   │   │   ├── TimesheetTable/
│   │   │   │   │   ├── TimesheetTable.view.svelte  # Weekly grid: project rows, comment sub-rows,
│   │   │   │   │   │                               # Mon–Sun columns, Total column and footer
│   │   │   │   │   ├── TimesheetTable.styles.ts
│   │   │   │   │   ├── TimesheetTable.hooks.ts     # Cell formatting, zero detection, column widths
│   │   │   │   │   └── index.ts
│   │   │   │   ├── RangeSelector.view.svelte       # Full / Yesterday + today / custom range
│   │   │   │   ├── RoundingToggle.view.svelte      # "Round to 0.5h" toggle
│   │   │   │   ├── GeneratedTimestamp.view.svelte  # "Generated at …" + Update now
│   │   │   │   └── ExportButton.view.svelte        # Export to Excel button
│   │   │   │
│   │   │   ├── settings/
│   │   │   │   ├── SettingsPanel.view.svelte       # Settings section of QuickPanel
│   │   │   │   ├── OpacitySlider.view.svelte       # Opacity label + slider + percentage
│   │   │   │   └── CompactModeToggle.view.svelte
│   │   │   │
│   │   │   └── diagnostics/
│   │   │       └── DiagnosticsPanel.view.svelte    # Storage paths, version, migration status
│   │   │
│   │   └── screens/                                # Screens are thin composers
│   │       │                                       # They instantiate controllers, read stores,
│   │       │                                       # and pass state + callbacks to domain components.
│   │       ├── QuickPanelScreen.svelte             # Primary floating window
│   │       ├── TimesheetScreen.svelte              # Dedicated timesheet preview window
│   │       └── DiagnosticsScreen.svelte            # Diagnostic log viewer
│   │
│   ├── services/
│   │   └── bridge/                                 # Typed Tauri command clients
│   │       │                                       # Called by controllers only.
│   │       │                                       # Each file wraps invoke() for one domain.
│   │       │                                       # No business rules live here.
│   │       ├── projectBridge.ts                    # add, remove, reorder, select, list
│   │       ├── sessionBridge.ts                    # startTracking, stopTracking, setComment
│   │       ├── timesheetBridge.ts                  # generate, getPreview, exportToExcel, reset
│   │       ├── settingsBridge.ts                   # load, save, applyOpacity, setAlwaysOnTop
│   │       └── diagnosticsBridge.ts                # getReport, openLogFile
│   │
│   ├── stores/                                     # Svelte stores — written by controllers, read by views
│   │   │                                           # and screens. Never written to directly from views.
│   │   ├── projectStore.ts                         # Ordered project list, active project, sort mode
│   │   ├── sessionStore.ts                         # Active session state, current comment text
│   │   ├── timesheetStore.ts                       # Preview rows, selected range, rounding state
│   │   └── settingsStore.ts                        # App settings, opacity, compact mode
│   │
│   └── utils/                                      # Pure stateless helpers — no MVC imports
│       ├── formatters.ts                           # Duration display, date formatting, relative time
│       └── sanitize.ts                             # Project name trimming and validation helpers
│
│
├── src-tauri/src/                                  # Native (Rust)
│   │
│   ├── commands/                                   # Tauri transport endpoints — no domain logic
│   │   │                                           # A command that contains business logic is a bug.
│   │   ├── project_commands.rs
│   │   ├── session_commands.rs
│   │   ├── timesheet_commands.rs
│   │   ├── settings_commands.rs
│   │   └── diagnostics_commands.rs
│   │
│   ├── controllers/                                # [CONTROLLER] Domain rules and workflow orchestration
│   │   ├── project_controller.rs                  # Add, remove, reorder, select; enforce name uniqueness;
│   │   │                                           # update recent usage metadata
│   │   ├── session_controller.rs                  # Start/stop tracking; write log entry on transition;
│   │   │                                           # attach comment to active session block
│   │   ├── timesheet_controller.rs                # Delegate to timesheet_service; apply rounding;
│   │   │                                           # coordinate export; enforce no-mutation-of-source rule
│   │   ├── settings_controller.rs                 # Load, merge, validate, and persist settings;
│   │   │                                           # notify shell of opacity/always-on-top changes
│   │   ├── shell_controller.rs                    # Window show/hide, position restore, tray sync,
│   │   │                                           # compact mode, autostart toggle
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
│   ├── repositories/                               # Concrete persistence — one file per entity
│   │   │                                           # Controllers never know the storage format.
│   │   ├── file_project_repository.rs             # Implements ProjectRepository over local file
│   │   ├── file_log_repository.rs                 # Implements LogRepository; append-only log format
│   │   └── file_settings_repository.rs            # Implements SettingsRepository over local config file
│   │
│   ├── services/                                   # Domain services — called by controllers, not commands
│   │   ├── timesheet_service.rs                   # Aggregates log entries → per-project daily hour totals;
│   │   │                                           # applies rounding; builds preview rows and comment sub-rows;
│   │   │                                           # produces export-ready data; never mutates source log
│   │   ├── migration_service.rs                   # Startup detection of legacy file formats;
│   │   │                                           # upgrades format in place; records migration in diagnostic log
│   │   └── export_service.rs                      # Serializes timesheet DTO to Excel (.xlsx)
│   │
│   ├── infrastructure/                             # OS-level glue — no domain logic lives here
│   │   ├── tray.rs                                # Tray menu construction and event dispatch;
│   │   │                                           # maps tray events to controller calls immediately;
│   │   │                                           # syncs item enabled/disabled state from active session
│   │   ├── window.rs                              # Window creation, show/hide, bounds save/restore,
│   │   │                                           # always-on-top, opacity application
│   │   └── autostart.rs                           # OS autostart registry integration
│   │
│   └── state.rs                                    # AppState — wires all controllers into Tauri manage();
│                                                   # single shared instance across commands and tray handlers
│
│
├── tests/
│   ├── e2e/                                        # Playwright end-to-end tests
│   │   ├── quick_track.spec.ts                     # Fast project switching in QuickPanel
│   │   ├── comment_attach.spec.ts                  # Comment saved against correct session
│   │   ├── timesheet_preview.spec.ts               # Daily hour totals are accurate
│   │   ├── timesheet_rounding.spec.ts              # Rounding rules applied correctly
│   │   ├── tray_project_select.spec.ts             # Tray selection updates QuickPanel
│   │   ├── settings_persist.spec.ts                # Opacity, compact mode survive restart
│   │   └── export_excel.spec.ts                   # Export produces valid .xlsx without corrupting log
│   │
│   └── rust/                                       # Cargo integration tests
│       ├── timesheet_aggregation.rs               # Multi-day log → correct daily totals
│       ├── timesheet_rounding.rs                  # Edge cases: sub-interval, overnight, zero entries
│       └── migration.rs                           # Legacy format detected, upgraded, re-readable
│
│
├── scripts/
├── docs/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── svelte.config.js
├── playwright.config.ts
└── README.md
```

---

## Naming Conventions

| Artefact                 | Convention              | Example                          |
| ------------------------ | ----------------------- | -------------------------------- |
| Domain type file         | `{entity}.types.ts`     | `session.types.ts`               |
| Validation schema        | `{Entity}Schema.ts`     | `SettingsSchema.ts`              |
| Frontend controller      | `create{Domain}Controller.ts` | `createTimesheetController.ts` |
| Bridge service           | `{domain}Bridge.ts`     | `timesheetBridge.ts`             |
| Svelte store             | `{domain}Store.ts`      | `projectStore.ts`                |
| View component file      | `{Name}.view.svelte`    | `ProjectRow.view.svelte`         |
| Component styles         | `{Name}.styles.ts`      | `TimesheetTable.styles.ts`       |
| Component hooks          | `{Name}.hooks.ts`       | `TimesheetTable.hooks.ts`        |
| Screen component         | `{Name}Screen.svelte`   | `TimesheetScreen.svelte`         |
| Rust controller          | `{domain}_controller.rs`| `timesheet_controller.rs`        |
| Repository trait         | `{domain}_repository.rs`| `log_repository.rs`              |
| Concrete repository      | `file_{domain}_repository.rs` | `file_log_repository.rs`   |
| Domain model             | `{entity}.rs`           | `log_entry.rs`                   |
| DTO                      | `{domain}_dto.rs`       | `timesheet_dto.rs`               |
| Tauri command file       | `{domain}_commands.rs`  | `session_commands.rs`            |

Folder names use `kebab-case` on the frontend and `snake_case` on the native side.

---

## Import Graph Rule

Each layer may only import from layers below it. Violations are bugs.

### Frontend

```
┌──────────────────────────────────────────┐
│  Screens                                 │  imports controllers, stores, view components
├──────────────────────────────────────────┤
│  View components                         │  imports ui primitives, utils — nothing else
├──────────────────────────────────────────┤
│  Controllers                             │  imports bridge services, stores, models/types
├──────────────────────────────────────────┤
│  Bridge services / Stores                │  imports models/types only
├──────────────────────────────────────────┤
│  Models / schemas / utils                │  no MVC imports
└──────────────────────────────────────────┘
```

Screens are the only layer permitted to read from stores and instantiate controllers. View components never read from stores — they receive state as props.

### Native

```
┌──────────────────────────────────────────┐
│  Commands / Tray handlers                │  imports controllers and state only
├──────────────────────────────────────────┤
│  Controllers                             │  imports repositories, services, models, state
├──────────────────────────────────────────┤
│  Repositories / Services                 │  imports models, infrastructure, utils
├──────────────────────────────────────────┤
│  Models / DTOs / traits                  │  no UI concerns, no OS concerns
└──────────────────────────────────────────┘
```

---

## Component Decomposition Convention

Every non-trivial view component splits into up to three files in its own folder:

```
ProjectList/
├── ProjectList.view.svelte   ← markup and prop destructuring only
├── ProjectList.styles.ts     ← all style objects
├── ProjectList.hooks.ts      ← formatting, local state, derived values
└── index.ts                  ← re-exports the view component
```

**`*.view.svelte`** — contains only markup and prop destructuring. No conditional logic beyond template branching. No format calls. No local `$state`. Calls to handlers are passed in as callbacks via `$props()`.

**`*.styles.ts`** — exports plain style constant objects. No imports from the MVC layers. No reactive code.

**`*.hooks.ts`** — exports a `create{Name}Hooks` function that takes props and returns derived values and formatting helpers. Uses `$derived` for reactive derivations. No side effects. No store writes.

Simple presentational components (e.g. `SortTabs`, `RoundingToggle`) that need no formatting or local state may use a single file without a folder.

---

## Frontend Controller Pattern

Controllers are the only layer screen components interact with. They own async state, call bridge services, normalize data, write to stores, and expose a stable action surface. They never render markup.

```ts
// src/controllers/projects/createProjectListController.ts
export function createProjectListController() {
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function selectProject(name: string) {
    loading = true;
    error = null;
    try {
      const nextState = await projectBridge.selectProject(name);
      projectStore.apply(nextState); // write-through: only on success
    } catch {
      error = "Could not select project.";
    } finally {
      loading = false;
    }
  }

  return {
    selectProject,
    get loading() { return loading; },
    get error()   { return error; },
  };
}
```

**What a controller owns:**
- Local `loading` and `error` `$state`
- All business rules for its domain (validation, sequencing, error handling)
- Calls to bridge services
- Store writes (only after side effects succeed)
- The action surface returned to screens

**What a controller never does:**
- Returns or imports JSX/Svelte markup
- Imports from other controllers
- Calls `invoke` directly (delegates to bridge)
- Writes to a store before the native side effect succeeds

---

## Store Pattern

Stores hold committed domain state. They are written by controllers and read by screens and view hooks. View components never read from stores directly — they receive store-derived state as props from screens.

```ts
// src/stores/projectStore.ts
import { writable, derived } from 'svelte/store';
import type { ProjectState } from '../models/types';

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectState | null>(null);

  return {
    subscribe,
    apply: (state: ProjectState) => set(state),
    reset: () => set(null),
  };
}

export const projectStore = createProjectStore();
```

**Store rules:**
- One store per domain slice
- Stores expose a minimal mutation surface (`apply`, `reset`) — not a generic `set`
- Controllers call `apply` only after the native side effect succeeds
- Transient state (loading flags, draft values, dialog states) lives in controller `$state`, not in stores
- Derived values (e.g. sorted project list, active project name) use Svelte `derived()` inside the store or in component hooks — not recomputed inline in views

---

## Native Controller Pattern

Native controllers own application rules. Tauri commands and tray handlers forward to them immediately — neither is a substitute for a controller.

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

---

## Repository Pattern

Repositories abstract persistence entirely. Controllers never know the storage format.

```rust
// src-tauri/src/models/repository_traits/log_repository.rs
pub trait LogRepository {
    fn append(&self, entry: LogEntry) -> Result<(), String>;
    fn list_for_day(&self, day: NaiveDate) -> Result<Vec<LogEntry>, String>;
    fn list_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<LogEntry>, String>;
}
```

Swapping the storage format (flat file → structured format) requires changing only the concrete repository in `repositories/`, not the controller.

---

## Command Boundary Pattern

Tauri commands are transport adapters only. If a command contains business logic, that logic belongs in a controller.

```rust
// src-tauri/src/commands/session_commands.rs
#[tauri::command]
pub fn start_tracking(project: String, state: State<AppState>) -> Result<ProjectStateDto, String> {
    state.session_controller.start_tracking(&project)
}
```

---

## Tray Handler Pattern

Tray handlers are event adapters. They mirror the command boundary pattern: receive an event, delegate to a controller immediately, optionally sync the QuickPanel window.

```rust
// src-tauri/src/infrastructure/tray.rs
SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
    project if known_projects.contains(project) => {
        app.state::<AppState>().session_controller.start_tracking(project).ok();
        sync_quickpanel_if_open(&app);
    }
    "open_quickpanel" => {
        app.state::<AppState>().shell_controller.show_quickpanel();
    }
    _ => {}
}
```

---

## View Pattern

Views receive already-prepared state and emit intent via callbacks. A view knows what to show, but never what the business rules are.

```svelte
<!-- src/views/components/projects/ProjectRow.view.svelte -->
<script lang="ts">
  let { name, isActive, onSelect, onRemove }: {
    name: string;
    isActive: boolean;
    onSelect: (name: string) => void;
    onRemove: (name: string) => void;
  } = $props();
</script>

<div class="row" class:active={isActive}>
  <button onclick={() => onSelect(name)}>{name}</button>
  <button onclick={() => onRemove(name)}>×</button>
</div>
```

---

## Self-Documenting Folder Rule

A folder should answer one of these questions on sight:

| Question                        | Folder                        |
| ------------------------------- | ----------------------------- |
| Is this domain shape?           | `models/`                     |
| Is this business behavior?      | `controllers/`                |
| Is this rendering?              | `views/`                      |
| Is this transport?              | `services/bridge/` / `commands/` |
| Is this persistence detail?     | `repositories/`               |
| Is this OS-level glue?          | `infrastructure/`             |
| Is this shared reactive state?  | `stores/`                     |

If a file cannot be placed confidently, that usually means the responsibility is still unclear.

---

## Testing Philosophy

The MVC split makes each layer independently testable with small, focused mocks.

| Layer                     | What to test                                              | Mock boundary                       |
| ------------------------- | --------------------------------------------------------- | ----------------------------------- |
| Frontend `models/schemas` | Payload parsing and invalid-state rejection               | None needed                         |
| Frontend controllers      | Async state, command sequencing, error handling           | Mock bridge services                |
| Frontend views            | Render behavior, callbacks, keyboard and input flows      | Props only — no stores or bridges   |
| Bridge services           | Command normalization and return type mapping             | Mock `invoke`                       |
| Native controllers        | Domain workflows and rule enforcement                     | Mock repositories and services      |
| Native repositories       | File/storage behavior and data mapping                    | Mock filesystem / infrastructure    |
| `timesheet_service`       | Hour aggregation, rounding, comment grouping, range logic | Mock repositories or clock          |
| `migration_service`       | Legacy format detection, upgrade correctness              | Mock filesystem                     |
| End-to-end                | Real desktop behavior across both surfaces                | Playwright                          |

**Critical regression paths:**

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

1. User project data, comments, timestamps, and settings are stored locally only — no cloud, no auth.
2. The log is append-only. Timesheet generation reads from it; nothing writes to it except session transitions.
3. Persistence format is an implementation detail hidden behind repository traits.
4. The app must tolerate missing, partial, or legacy files through startup migration services.
5. Timesheet generation must be fully reproducible from persisted log data at any time.
6. Export logic must never mutate source log history.

---

## Commenting Practice

Prefer self-explanatory names over explanatory comments. Add a comment only where it conveys something the code itself cannot: an architectural constraint, a non-obvious invariant, a workaround for a specific bug, or surprising behavior. Comment the *why*, not the *what*.

- Comment invariants, assumptions, and surprising behavior.
- Comment cross-layer or cross-domain decisions that would be hard to infer locally.
- Never narrate what the next line does.

---

## Refactor Guidance

During any refactor pass, one file at a time:

- Do not move business logic into Svelte views
- Do not let Tauri commands become controller substitutes
- Do not let tray handlers contain domain logic
- Do not hide domain rules inside utility files
- Do not let desktop-shell code leak into project/session/timesheet domains
- Do not optimize for current file layout over long-term clarity

When a refactor pass breaks downstream files intentionally, state that clearly. The app is allowed to be broken between layer passes — the agent working the refactor must not leave backwards-compatibility shims behind to avoid that breakage. Make the clean change and document what must follow.

---

## Summary

ProjectLog is a dual-surface MVC desktop application:

- **Frontend MVC** — Svelte 5 rendering and interaction in the QuickPanel and Timesheet windows
- **Native MVC** — Rust domain logic, persistence, tray, and OS integration
- **Thin transport boundary** — typed bridge services and Tauri commands connect the two
- **Two surfaces, one domain** — QuickPanel and tray both call the same controllers and reflect the same committed state

That structure serves one mission: a fast, private, local-only desktop tool for logging what was worked on each day, how long it took, and turning that activity into reliable project-hour output.
