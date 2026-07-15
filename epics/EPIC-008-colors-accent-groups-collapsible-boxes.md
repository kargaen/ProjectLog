# EPIC-008: Colors as left accent, groups as collapsible boxes

**Status:** active
**Created:** 2026-07-15
**Architecture baseline:** aef005e

Redesigns the QuickPanel project color/grouping UI toward two user mockups: color as a
vertical left-edge accent bar before the text, and groups as bordered collapsible boxes with
indented members. Intentionally **supersedes** EPIC-006's shipped rendering — Flow A (color as
underline strip) and Flow D (flat "named groups above ungrouped" list) are replaced. Also
brings groups to the system tray as submenus.

---

## 1. BDD — User Flows

### Flow 1: Color as a left accent bar

```gherkin
Given a project has an assigned color
When the project list renders
Then a vertical bar of that color sits at the left edge of the project's row, before the text
And there is no background fill and no underline on the row
```

### Flow 2: Groups render as collapsible boxes with indented members

```gherkin
Given grouping is shown and some projects are assigned to named groups
When the project list renders
Then each named group appears as a bordered box with a header showing a collapse/expand chevron and the group name
And that group's member projects appear indented inside the box
And ungrouped projects appear with no box and no indentation
And a group with no member projects is not shown at all (see Flow 11 for where empty group names still appear)
```

### Flow 3: Collapsing a group hides its members, visually only

```gherkin
Given a group box with visible members
When the user clicks the group's collapse chevron
Then the group's member projects are hidden and the chevron shows the collapsed state
And no project's group membership, order, or color changes
When the user clicks the chevron again
Then the members are shown again
```

### Flow 4: A checkbox toggles whether groups are shown

```gherkin
Given at least one project is assigned to a group
When the user views the sort row (Manual / A-Z / Recent)
Then the grouping control is a checkbox (not a button)
When the checkbox is unchecked
Then the list is flat (no boxes, no indentation)
When it is checked
Then the list shows group boxes per Flow 2

Given no project is assigned to any group
When the user views the sort row
Then the grouping checkbox is not shown at all

Given Manual sort is selected
When the user views the grouping checkbox
Then it is checked and disabled (locked on), because manual drag reordering operates within groups
And hovering it reveals that manual mode requires groups to be enabled
And leaving Manual restores the checkbox to a normal toggle
```

### Flow 5: Acting on a group forces grouping on

```gherkin
Given grouping is currently not shown
When the user creates a new group or assigns a project to a group via the context menu
Then grouping is forced on (the checkbox becomes checked)
And the resulting grouped layout is shown immediately
```

### Flow 6: A-Z sorts groups and ungrouped together, then members

```gherkin
Given grouping is shown and A-Z is selected
When the list renders
Then level one — group names and ungrouped project names together — is ordered A-Z
And within each group, level two — the member project names — is ordered A-Z
```

### Flow 7: Recent orders groups by their most recent member

```gherkin
Given grouping is shown and Recent is selected
When the list renders
Then level one is ordered by recency, where a group's recency is the most recent among its members and an ungrouped project uses its own
And within each group, members are ordered by recency
```

### Flow 8: Manual drag reorders only within a group

```gherkin
Given grouping is shown and Manual is selected
When the user drags a project within its own group (or within the ungrouped set)
Then the project reorders among its siblings there
When the user drags a project toward a different group or outside grouping
Then it does not move between groups or in/out of grouping
And moving a project to another group or out of all groups is available only through the context menu
```

### Flow 9: System tray shows groups as submenus (native)

```gherkin
Given projects are assigned to named groups
When the system tray menu is opened
Then each named group is a submenu containing its member projects
And ungrouped projects appear at the top level of the menu
And selecting a project inside a group submenu activates it exactly as a top-level project does
When the user changes group membership in the QuickPanel
Then the tray menu reflects the new grouping the next time it is built
```

### Flow 10: Every project has a timestamp from creation

```gherkin
Given a project is created (added or quick-tracked) and has never been selected
When the list is ordered by Recent
Then the project still has a recency timestamp (its creation time), so it takes a definite position
And no project is ever missing a timestamp
```

### Flow 11: Empty group names remain pickable in the context menu

```gherkin
Given a group name exists but currently has no member projects
When the user opens a project's context menu to assign a group
Then that group name still appears as a pickable option
And the empty group is not rendered as a box in the project list (Flow 2)
```

**Out of scope for this epic:**
- Colors or collapse in the tray — the native menu shows neither. The tray also cannot set a
  project's color, group, or collapsed state; it only selects/activates.
- Subgroups / nested groups — exactly one level of grouping (a group contains projects, never
  other groups).
- Changing group membership by drag — cross-group and un-group moves are context-menu only.
- Renaming or deleting a group as a first-class action, beyond what re-assigning members
  already implies (unless it falls out for free; not specified here).
- The color palette, the context menu's placement/clamping (EPIC-006), and timesheet/logging
  behavior.

---

## 2. Function Call Signatures

Two contracts constrain work across files and are worth pinning; the rest are markup/CSS.

```ts
// src/lib/ — pure, unit-tested (sibling to menuPosition.ts)
// Build the two-level grouped view model from flat inputs. Ordering per `mode`.
// Level 1 mixes group entries and ungrouped project entries; groups carry their members.
type GroupedView = Array<
  | { kind: "group"; name: string; projects: string[] }
  | { kind: "project"; name: string }        // ungrouped, level 1
>;
function buildGroupedView(
  orderedProjects: string[],                  // already in the base/manual order
  groups: Record<string, string>,             // project -> group name
  mode: "manual" | "alphabetical" | "recent",
  recentUsage: Record<string, number>,        // project -> timestamp
): GroupedView;
```

```rust
// src-tauri/src/infrastructure/tray_menu.rs — the group-derived tray structure.
// Pure enough to unit-test: (projects, groups, sort) -> ordered tree of top-level items
// and group submenus. Select ids are keyed by project name / a stable key (Q3), NOT by a
// positional index, so selection resolves regardless of submenu nesting.
```

The collapse toggle, the checkbox, and the accent-bar markup constrain nothing downstream —
no signatures.

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Function / surface under test | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| 1 | rendered row | Playwright DOM — a `.project-color-accent` (or agreed class) element exists at the row's left with the assigned color; no background-fill / underline | `installTauriMocks` with a colored project | exact — accent background equals the hex; row background unchanged |
| 2 | rendered list | Playwright DOM — group box element wraps a header (chevron + name) and indented member rows; ungrouped rows have neither box nor indent class | mock state with named groups + ungrouped | exact — structure/classes present |
| 3 | collapse behavior | Playwright DOM — after clicking the chevron, member rows are absent/hidden; group membership + order unchanged; re-click restores | grouped mock state | exact — members hidden then shown; no state mutation observed via a follow-up get_state |
| 4 | grouping control | Playwright DOM — control is a checkbox; toggling switches between flat and boxed layouts; hidden when no groups exist; in Manual it is checked + disabled with a hover hint | grouped mock state, and a no-groups mock state | exact |
| 5 | force-enable | Playwright — with grouping off, invoking new-group / assign-group leaves grouping on and boxed | grouped mock state, grouping initially off | exact — group_projects_enabled true afterward |
| 6 | `buildGroupedView(mode: "alphabetical")` | Textbook (lexicographic order) + hand-computed expected tree | unit fixture: groups + ungrouped with known names | exact — level-1 and level-2 sequences match |
| 7 | `buildGroupedView(mode: "recent")` | Definition T_group = max(member timestamps); hand-computed expected tree | unit fixture with known timestamps | exact — level-1 by T_group, level-2 by member recency |
| 8 | drag within group | Playwright — dragging within a group reorders siblings; a drag targeting another group/outside does not change membership | grouped manual-mode mock state | exact — membership unchanged; sibling order changed only within group |
| 9 | tray group tree | Rust unit test — (projects, groups, sort) → expected ordered top-level items + per-group submenu contents with name-keyed select ids | Rust fixture: projects + group map | exact — tree shape, contents, and id resolution |
| 10 | project creation timestamp | Rust test — adding / quick-tracking a project records a recency timestamp so Recent has a value before any selection | Rust fixture: add a project, read `project_recent_usage` | exact — the new project has a non-empty timestamp |
| 11 | empty group in picker | Playwright DOM — a group with no members shows no box in the list but its name is offered in the context menu group picker | mock state with a named-but-memberless group | exact — no box rendered; name present in picker |

`buildGroupedView` (flows 6/7, and the shape behind 2/8) is the load-bearing pure function —
unit-tested like `clampMenuPosition`. The tray tree builder (flow 9) is the load-bearing
native contract.

### What is deliberately not tested

- OS-level menu rendering and native submenu appearance (flow 9) — manual verification only.
- Exact pixel width of the accent bar, indentation depth, box border styling — visual, not
  asserted beyond presence/position.
- Collapse-state persistence — by decision (Q1) collapse is ephemeral and resets to expanded
  on restart; there is nothing persisted, so nothing to test there. Tests assert in-session
  collapse behavior only.
- Drag animation / drop-indicator visuals — only the resulting order/membership is asserted.

---

## 4. Checklist

Natural slice boundaries, in dependency order. Later items depend on the view model (items
1–2). Tests precede the code they pin. Items marked (blocked …) wait on an open question.

```md
[ ] 1. Add failing unit tests for `buildGroupedView` A-Z + Recent two-level ordering (flows 6/7) in a new test file under `tests/ui/` — done when they fail for the right reason
[ ] 2. Implement `buildGroupedView` in `src/lib/groupedView.ts` — done when item 1 passes
[ ] 3. Add failing Playwright test for the color accent bar (flow 1) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 4. Replace the underline with a left accent bar in `src/views/components/projects/ProjectListPanel.view.svelte` — done when item 3's DOM assertions pass
[ ] 5. Move accent-bar styling into `src/views/screens/quickpanel.css` (remove `.project-color-underline`) — done when item 3 fully passes
[ ] 6. Add failing Playwright test for group boxes + indentation + ungrouped-no-box (flow 2) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 7. Render group boxes from `buildGroupedView` in `src/views/components/projects/ProjectListPanel.view.svelte` — done when item 6 passes
[ ] 8. Add group-box / indentation / chevron styles in `src/views/screens/quickpanel.css` — done when item 6 fully passes
[ ] 9. Wire the controller to expose the `buildGroupedView` model in `src/controllers/quickpanel/createQuickPanelController.svelte.ts` (replaces the old `groupedProjects` getter) — done when items 6/3 pass through the real controller
[ ] 10. Add failing Playwright test for collapse/expand (flow 3) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 11. Add ephemeral (non-persistent) collapse state + toggle in the QuickPanel controller `src/controllers/quickpanel/createQuickPanelController.svelte.ts` — done when item 10 passes
[ ] 12. Add failing Playwright test for the grouping checkbox (flow 4) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 13. Replace the "Group" button with a checkbox in `src/views/components/projects/ProjectListPanel.view.svelte` — done when item 12 passes
[ ] 14. Add failing Playwright test for force-enable on group action (flow 5) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 15. Force `group_projects_enabled` true on new-group/assign-group in `src/controllers/projects/createProjectContextMenuController.ts` — done when item 14 passes
[ ] 16. Add failing Playwright test for drag-locked-to-group (flow 8) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 17. Constrain drag reordering to within the item's group span in the flat `project_manual_order`, in `src/controllers/projects/createProjectActionsController.ts` — done when item 16 passes
[ ] 18. Add failing Rust unit test for the group→submenu tree with name-keyed select ids (flow 9) in `src-tauri/src/infrastructure/tray_menu.rs` (or a sibling module) — done when it fails for the right reason
[ ] 19. Build group submenus with name-keyed (not index-keyed) select ids in `src-tauri/src/infrastructure/tray_menu.rs` — done when item 18 passes
[ ] 20. Add failing Playwright test: grouping checkbox is hidden with no groups, and locked checked + disabled with a hover hint in Manual (flow 4) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 21. Implement checkbox visibility (hidden when no groups) and Manual-mode lock + tooltip in `src/views/components/projects/ProjectListPanel.view.svelte` — done when item 20 passes
[ ] 22. Add failing Playwright test: an empty group renders no box in the list but its name is still offered in the context menu group picker (flows 2/11) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[ ] 23. Hide empty-group boxes in the grouped view while keeping empty group names in the picker — `src/lib/groupedView.ts` (list) — done when item 22's list assertion passes
[ ] 24. Add failing Rust test then record a creation timestamp when a project is added/quick-tracked so Recent always has a value (flow 10), in the native project controller `src-tauri/src/controllers/project_controller.rs` — done when the test passes
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [x] Amends Description sections: `description/13-project-list-color-grouping.md` (via `epic-closeout` after it ships) — color rendering, grouped rendering, tray submenus. No new settings field: collapse is ephemeral (Q1), and the creation timestamp (Flow 10) writes into the existing `project_recent_usage`.
- [ ] **Requires a Constitution change**

No Constitution change. This is the "two surfaces, one domain" decision (constitution/01 #5,
description/02) exercised — QuickPanel and tray both reflect the same group state; neither
owns it. **Supersedes EPIC-006 Flow A (underline) and Flow D (flat group order)** — those
shipped behaviors are intentionally replaced here, not regressions.

### North star deviation

North star: *"a frictionless time-tracking companion… tell you exactly how much time you spend
on what project several weeks back… when you have seventeen ongoing projects."*

**No.** This touches only how the project list is organized and shown; it changes nothing
about logging, sessions, or timesheet reconstruction. It directly serves the seventeen-projects
scenario — collapsible groups and a scannable colored list are the at-a-glance organization
that case needs. Nothing about trust or accuracy is traded.

### Open questions — resolved (2026-07-15)

| # | Decision |
|---|---|
| Q1 | **Collapse state is not persistent.** It is ephemeral QuickPanel controller state — no settings field, resets to expanded on restart. Item 11 is unblocked. |
| Q2 | **Manual mode keeps groups shown.** In Manual sort the grouping checkbox is locked checked + disabled (Flow 4), with a hover hint that manual mode requires groups; drag reorders only within a group. Rationale: manual drag operates on the grouped view, so grouping cannot be toggled off there. *Residual implementation detail (not blocking review):* the flat `project_manual_order` still stores the order — drag bounds movement to the item's group span within that flat list; how level-1 (group vs group / vs ungrouped) is ordered in Manual is settled in the drag slice (item 17). |
| Q3 | **Use stable keys, not indices.** Tray `select::` ids key on the project name / a stable key rather than a positional index, so selection resolves regardless of submenu nesting. Item 19 is unblocked on this basis. |
| Q4 | **Every project always has a timestamp (creation time).** Projects get a recency timestamp when created (Flow 10), so a group never has timestamp-less members and `T_group = max(members)` is always defined. Empty groups do not appear in the list at all (Flow 2 / Flow 11), so they never need level-1 placement. |

### New capability

Collapse/expand of groups is genuinely new (the current feature has no collapse). Group
submenus in the tray are new. Both are organization affordances the north star's
"seventeen ongoing projects" implies but never spelled out — named here so the scope
expansion is explicit, not silent.

### Note on size

This is a large epic (nine flows, both surfaces). The flows share one feature, one fixture
family, and the color/group files, so it is one epic, not two — but the checklist is grouped
so it can be shipped in slices (color accent → group boxes → collapse → checkbox/force-enable
→ two-level sorts → drag-lock → tray submenus). If you prefer, it can be split into sub-epics
along those boundaries at review time; say so and I will carve it.
