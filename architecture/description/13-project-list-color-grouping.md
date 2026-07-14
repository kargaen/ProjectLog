## Project List — Color & Grouping

The QuickPanel project list lets each project carry a color and an optional group. Both are per-user settings keyed by project name, following the same shape as `project_recent_usage`:

- `project_colors` — `Record<project name, hex string>` (`src/models/types/project.ts`; `HashMap<String, String>` in `src-tauri/src/models/domain/settings.rs`).
- `project_groups` — `Record<project name, group name>`; a project absent from the map is ungrouped.
- `group_projects_enabled` — bool; when false the list renders flat.

### Assignment

Right-clicking a project row opens `src/views/components/projects/ProjectContextMenu.view.svelte`, driven by `src/controllers/projects/createProjectContextMenuController.ts`. Picking a swatch or a group calls one of two native commands:

- `set_project_color(project, color: Option<String>)`
- `set_project_group(project, group: Option<String>)`

Command wrappers live in `src-tauri/src/commands/settings_commands.rs`; the logic — insert on `Some`, remove the entry on `None`, persist, emit a state change — in `src-tauri/src/controllers/settings_controller.rs::set_project_color` / `set_project_group`; both are registered in `src-tauri/src/lib.rs` and reached from the frontend through `settingsBridge.setProjectColor` / `setProjectGroup`. Choosing "New group…" reuses the shared input dialog via the `new_group::` dialog-mode prefix.

### Rendering

- An assigned color renders as a full-width strip of the true color (`.project-color-underline`) along the bottom of the project row — not a background fill on the title box or the remove/save (×/＋) icon button.
- When `group_projects_enabled`, the `groupedProjects` getter in `src/controllers/quickpanel/createQuickPanelController.svelte.ts` buckets the sorted project list by group and returns named groups (A–Z, each under a header) first, then the ungrouped bucket last with no header.

### Context-menu placement

`src/lib/menuPosition.ts` exports the pure function `clampMenuPosition(click, menu, viewport)`, which returns the top-left at which the menu sits fully inside the viewport — pulling it flush with an overflowed right or bottom edge, and pinning to the top-left corner when the menu is larger than the viewport. `ProjectContextMenu.view.svelte` measures its own rendered `offsetWidth`/`offsetHeight` and the viewport, then applies the clamped position so a right-click near a window edge does not clip the menu.
