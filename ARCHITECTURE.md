---
## Mission

See [`README.md`](./README.md) for the full mission statement and product overview.

---

## North Star

ProjectLog is a frictionless time-tracking companion: always there, never in the way — a clock punch card that follows you through the day without ever interrupting it.

When the phone rings about an important project you weren't expecting, while you were deep in something else, ProjectLog is ready to mark the moment the call started — just by saying the word.

When you're juggling seventeen ongoing projects and your boss wants that overdue timesheet, ProjectLog can tell you, with surgical accuracy, exactly how much time you spent on what, weeks back.

Every decision in this document exists to protect that: minimal friction to log a moment, and unshakeable trust in what gets reconstructed from it later. An epic that trades either away — that adds friction to the moment of logging, or weakens the reconstructed record's accuracy — erodes the north star and must say so plainly.

---

## Write Policy

This document is sharded across `architecture/constitution/` and `architecture/description/`.

- **`constitution/`** — human-reviewed principles, conventions, and agent working rules. Agents may cite these files but never edit them. Changes go through `architecture-md-maintenance` with explicit user confirmation.
- **`description/`** — reactive documentation of what the code actually does (structure, stack, patterns). Written only by `epic-closeout` after a slice ships.

If a change would touch `description/` before the corresponding code exists, that content belongs in an epic, not here.

---

## Index

| File | Contains |
|---|---|
| `constitution/01-five-design-decisions.md` | The five load-bearing design decisions |
| `constitution/02-architecture-philosophy.md` | MVC split, frontend/native boundary |
| `constitution/03-architectural-goals.md` | The six goals the architecture must serve |
| `constitution/04-data-flow.md` | One-directional data flow diagrams and hard rules |
| `constitution/05-naming-conventions.md` | File/folder naming table |
| `constitution/06-import-graph-rule.md` | Per-layer import direction, frontend and native |
| `constitution/07-component-decomposition-convention.md` | `.view.svelte` / `.styles.ts` / `.hooks.ts` split |
| `constitution/08-self-documenting-folder-rule.md` | Question → folder table |
| `constitution/09-testing-philosophy.md` | Layer → test-focus table, critical regression paths |
| `constitution/10-persistence-philosophy.md` | Local-only, append-only, reproducible persistence rules |
| `constitution/11-commenting-practice.md` | Comment-the-why convention |
| `constitution/12-refactor-guidance.md` | One-file-at-a-time refactor rules, no compat shims |
| `constitution/13-release-branch-model.md` | Branch roles, versioning constraint, RC definition |
| `description/01-stack.md` | Tech stack table |
| `description/02-two-surfaces-one-domain.md` | QuickPanel and tray feature lists |
| `description/03-domain-boundaries.md` | The six business domains |
| `description/04-directory-tree.md` | Full annotated repo directory tree |
| `description/05-frontend-controller-pattern.md` | Frontend controller code example |
| `description/06-store-pattern.md` | Svelte store code example |
| `description/07-native-controller-pattern.md` | Native controller code example |
| `description/08-repository-pattern.md` | Repository trait code example |
| `description/09-command-boundary-pattern.md` | Tauri command code example |
| `description/10-tray-handler-pattern.md` | Tray event handler code example |
| `description/11-view-pattern.md` | View component code example |
| `description/12-release-shipping-procedure.md` | Versioning mechanics, CHANGELOG.md lifecycle, shipping steps |
| `description/13-project-list-color-grouping.md` | Project color/group settings, context-menu assignment, grouped rendering, menu-placement helper |
| `25-change-history.md` | Append-only log of Description amendments (neither constitution nor description) |

---

## Summary

ProjectLog is a dual-surface MVC desktop application:

- **Frontend MVC** — Svelte 5 rendering and interaction in the QuickPanel and Timesheet windows
- **Native MVC** — Rust domain logic, persistence, tray, and OS integration
- **Thin transport boundary** — typed bridge services and Tauri commands connect the two
- **Two surfaces, one domain** — QuickPanel and tray both call the same controllers and reflect the same committed state

That structure serves one mission: a fast, private, local-only desktop tool for logging what was worked on each day, how long it took, and turning that activity into reliable project-hour output.
