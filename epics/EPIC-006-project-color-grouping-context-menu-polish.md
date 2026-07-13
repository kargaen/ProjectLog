# EPIC-006: Project color, grouping & context-menu polish

**Status:** active
**Created:** 2026-07-12
**Architecture baseline:** c7ac315

Finishes the project color/group context-menu feature, which shipped without an epic (plan
file `sequential-popping-valley.md`, no epic, not in `architecture/description/`, no tests).
`change-triage` routed it here: the color-as-background change is a new design decision with
no authority, and the whole feature has no existing authority to pin tests against.

---

## 1. BDD — User Flows

### Flow A: Color tints the project title box

```gherkin
Given a project has an assigned color
When the project list renders
Then the color fills the background of that project's title box
And the color does not touch the remove/save (×/＋) button beside it
And the project title stays left-aligned
```

### Flow B: Clearing a color restores the default

```gherkin
Given a project whose title box is tinted with a color
When the user opens its context menu and clears the color
Then the title box returns to the default (untinted) background
```

### Flow C: Context menu stays inside the window

```gherkin
Given the user right-clicks a project row near the QuickPanel's right or bottom edge
When the context menu opens
Then the whole menu is visible inside the QuickPanel window
And no part of it is clipped by the window edge
```

### Flow D: Named groups above ungrouped

```gherkin
Given grouping is enabled and some projects are assigned to named groups
When the project list renders
Then the named groups appear at the top, each under its own header
And the ungrouped projects appear below them
And the ungrouped projects have no header above them
```

**Out of scope for this epic:**
- The group-name creation dialog and color/group persistence — already implemented and
  working; this epic changes only presentation and menu placement.
- Documenting the color/group feature in `architecture/description/` — that is
  `epic-closeout`'s job once this lands (see §5), not a flow here.
- Tray-surface behavior — EPIC-002.
- Any new color preset or a custom color picker — the eight existing presets stand.

---

## 2. Function Call Signatures

Only Flow C introduces a contract worth pinning up front — the clamp is a pure function the
controller owns and the view consumes, and getting its shape wrong forces a rewrite of both.

```ts
// src/controllers/projects/createProjectContextMenuController.ts
// Given the raw click point, the menu's size, and the window's size,
// return the top-left at which the menu is fully inside the window.
function clampMenuPosition(
  click: { x: number; y: number },
  menu: { width: number; height: number },
  windowSize: { width: number; height: number },
): { x: number; y: number };
```

The other three flows are markup/CSS/ordering changes that constrain nothing downstream —
no signatures.

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Function under test | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| A | rendered project row | Playwright DOM assertion — the `.project-button` carries the color as its background; the `.icon-button` does not; computed `text-align` is left | `installTauriMocks` state with a colored project (`tests/ui/helpers/tauri.ts`) | exact — background color string equals the assigned hex; icon-button background unchanged |
| B | rendered project row after clear | Playwright DOM assertion — after clearing, `.project-button` background equals the default | same fixture, color then cleared | exact — background equals the untinted default |
| C | `clampMenuPosition` | Geometry — the returned rect lies entirely within `[0, windowSize]` on both axes, and equals the click point when the menu already fits | unit test, hand-constructed coordinate cases (fits; overflows right; overflows bottom; overflows both) | exact — `x + width ≤ windowSize.width`, `y + height ≤ windowSize.height`, `x ≥ 0`, `y ≥ 0` |
| D | `groupedProjects` bucket order | Playwright DOM assertion — group headers precede ungrouped rows in document order; existing `localeCompare` group sort preserved | `installTauriMocks` state with two named groups + ungrouped projects | exact — rendered bucket sequence is [named groups A–Z…, ungrouped] |

Flow C's clamp is unit-testable in isolation because it is a pure function of three inputs;
Flows A/B/D assert against the rendered DOM through the existing mock harness, which is the
only authority this UI has.

### What is deliberately not tested

- Exact pixel coordinates of the menu on real hardware — the clamp is verified as geometry,
  not as a screenshot.
- Contrast/legibility of title text over a tinted background — a visual-design judgement, not
  a testable invariant. If a color proves unreadable, that is a preset-palette change, not a
  regression against this epic.
- The color/group *persistence* round-trip — already covered by the feature as shipped;
  untouched here.

---

## 4. Checklist

Ordered by dependency. Flow A's item subsumes the old right-alignment bug (removing the dot
removes the `space-between` regression), so there is no separate item for it.

```md
[x] 1. Add failing Playwright test for Flow D (named groups above ungrouped) in `tests/ui/app.spec.ts` — done when it fails against current ungrouped-first order for the right reason
[x] 2. Swap bucket order in `groupedProjects` getter in `src/controllers/quickpanel/createQuickPanelController.svelte.ts` (named groups before ungrouped) — done when test 1 passes
[x] 3. Add failing Playwright test for Flow A (title box carries the color, icon-button does not, title left-aligned) in `tests/ui/app.spec.ts` — done when it fails for the right reason
[x] 4. Replace the color-dot span with a title-box background tint in `src/views/components/projects/ProjectListPanel.view.svelte` — done when test 3's DOM-structure assertions pass
[x] 5. Adjust `.project-button` / `.color-dot` rules in `src/views/screens/quickpanel.css` (remove dot styling, apply background tint, drop the `space-between` right-alignment side effect) — done when test 3 fully passes and Flow B (clear) is green
[x] 6. Add failing unit test for `clampMenuPosition` (fits / overflows right / bottom / both) in a new test file under `tests/ui/` — done when it fails for the right reason
[x] 7. Implement `clampMenuPosition` in ~~`src/controllers/projects/createProjectContextMenuController.ts`~~ `src/lib/menuPosition.ts` — done when test 6 passes
    - (amended 2026-07-12) Placed as a pure helper in `lib/`, not the controller: clamping needs the menu's rendered size, which only exists after mount, so the *view* measures and applies it — and a view may import a `lib` util but not a controller (import graph). `openContextMenu` keeps storing the raw click point; the clamp happens at render in item 8.
[x] 8. Consume `clampMenuPosition` in `src/views/components/projects/ProjectContextMenu.view.svelte` — view measures its own size (`offsetWidth/Height` via `bind:this`, so the 1px border counts) and the viewport (`<svelte:window>`), applies the clamped x/y — done when a Flow C Playwright check shows the menu fully within the window near an edge
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [x] Amends Description sections: none directly — but the color/group feature is still
  entirely undocumented in `architecture/description/` (drift-audit finding). Documenting it
  is `epic-closeout`'s job when this epic lands, covering both the feature and these
  refinements. This epic does not itself write to `description/`.
- [ ] **Requires a Constitution change**

No Constitution change required — nothing here touches a principle, boundary, or convention.

### North star deviation

North star: *"a frictionless time-tracking companion: always there, never in the way … minimal
friction to log a moment."*

**No.** A context menu clipped off-screen and a mis-ordered list are friction — the user
cannot pick a color they cannot see, and cannot scan a list that buries their groups. This
epic removes friction and trades nothing away. Coloring and grouping are presentation over
the same domain state; neither changes what is logged or reconstructed.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | The context menu's height is dynamic (grows with group count). Does clamp use a measured height (read from the DOM after mount, then reposition) or a conservative max estimate? | Blocks item 7 only | Slice 6/7 — pick measured-then-reposition if a fixed estimate visibly misplaces tall menus |
| Q2 | Two related fixes for this feature area exist unmerged on `claude/app-architecture-rewrite-j2daqd` (per EPIC-002 Q1). Confirm none overlaps these files before implementing, to avoid a later conflict. | No — check at slice start | Before slice 2 |

### New capability

None — this refines an existing feature; it introduces nothing the north star did not already
allude to.
