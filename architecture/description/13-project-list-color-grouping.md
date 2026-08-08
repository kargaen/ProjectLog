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

`src/lib/groupedView.ts` exports `buildGroupedView`, the shared two-level project-list view model. Level one mixes named group boxes and ungrouped project rows. A-Z mode sorts group names and ungrouped project names together, with members sorted A-Z inside each group. Recent mode sorts each group by its most recent member timestamp (`T_group = max(member timestamps)`) and sorts members by recency. Manual mode keeps the existing manual order and uses the first member position as the group position.

- An assigned color renders as a vertical left-edge accent bar (`.project-color-accent`) before the row text. Project rows no longer use a background fill or bottom underline for color.
- Adding or quick-tracking a project records an initial `project_recent_usage` timestamp, so Recent ordering has a value before the project is selected later.
- When grouping is enabled, `groupedProjects` in `src/controllers/quickpanel/createQuickPanelController.svelte.ts` maps `buildGroupedView` output into collapsible group entries. Collapse state is ephemeral frontend state only and is not persisted.
- `src/views/components/projects/ProjectListPanel.view.svelte` renders each non-empty group as a bordered `.project-group-box` with a chevron/header and indented `.group-member` project rows. Ungrouped projects render as normal top-level rows with no group box and no "Ungrouped" heading. Empty groups do not render boxes.
- The controller keeps group names seen in the current QuickPanel session available in the context-menu picker even after the last member is ungrouped, so an empty group can be reused without recreating it.
- The sort-row grouping control is a checkbox. It is hidden when no group name is known, and Manual sort mode forces grouping on and disables the checkbox because drag reordering is constrained to the current group span.

### Tray menu grouping

The native tray menu uses the same project color/group settings for structure, but not color or collapse UI. When `group_projects_enabled` is true, `src-tauri/src/infrastructure/tray_menu.rs` builds one submenu per non-empty group and leaves ungrouped projects as top-level menu items. Project selection IDs are derived from the project name rather than from positional indices so activation remains stable when items move into submenus or reorder.

### Context-menu placement

`src/lib/menuPosition.ts` exports the pure function `clampMenuPosition(click, menu, viewport)`, which returns the top-left at which the menu sits fully inside the viewport — pulling it flush with an overflowed right or bottom edge, and pinning to the top-left corner when the menu is larger than the viewport. `ProjectContextMenu.view.svelte` measures its own rendered `offsetWidth`/`offsetHeight` and the viewport, then applies the clamped position so a right-click near a window edge does not clip the menu.
