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
