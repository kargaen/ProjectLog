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
