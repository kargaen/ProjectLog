# Change History

Append-only. One row per Description amendment, written by `epic-closeout`. Constitution
changes are not recorded here.

| Date | Epic | Sections | Change |
|---|---|---|---|
| 2026-07-12 | EPIC-006 | `description/13` | Documented the project list color & grouping feature (color/group settings, context-menu assignment via `set_project_color`/`set_project_group`, grouped rendering order) and the new `src/lib/menuPosition.ts` context-menu placement helper. |
| 2026-07-13 | direct slice 59c06f8 | `description/13` | Project color now renders as a full-width bottom underline strip (`.project-color-underline`), not a title-box background fill. |
| 2026-07-13 | ci a189705, af4a8d2 | `description/12` | Release model reworked: `dev` push builds a rolling unversioned `rc` pre-release (`release-candidate.yml`); `master` push publishes the stable tagged release (`release.yml`, fail-loud on non-clean version, deletes the `rc`). Website stays with `pages.yml`. |
| 2026-07-16 | EPIC-008 | `description/02`, `description/13` | Replaced project color underline documentation with left-edge accents, documented the `buildGroupedView` two-level grouped rendering model, collapsible non-persistent group boxes, grouping checkbox/manual lock behavior, group-locked manual drag, and native tray group submenus with stable name-derived select IDs. |
| 2026-07-16 | EPIC-009 | `description/12` | RC version discipline: `release-candidate.yml` fails loudly when `package.json`'s base version is already published as a stable release; new `npm run bump -- patch\|minor\|major\|rc` (`scripts/bump.mjs`) computes the next numeric-suffix pre-release, syncs via `sync-version.mjs`, and commits without tagging or pushing. |
| 2026-07-17 | EPIC-008 | `description/13` | Added the creation-recency guarantee: newly added or quick-tracked projects receive an initial `project_recent_usage` timestamp so Recent ordering has a value before later selection. |
| 2026-07-17 | EPIC-008 | `description/13` | Closed the project color/grouping redesign: documented `groupedProjects`, empty groups staying hidden while known group names remain pickable, and the grouped checkbox/manual drag behavior as shipped. |
