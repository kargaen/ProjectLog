# Change History

Append-only. One row per Description amendment, written by `epic-closeout`. Constitution
changes are not recorded here.

| Date | Epic | Sections | Change |
|---|---|---|---|
| 2026-07-12 | EPIC-006 | `description/13` | Documented the project list color & grouping feature (color/group settings, context-menu assignment via `set_project_color`/`set_project_group`, grouped rendering order) and the new `src/lib/menuPosition.ts` context-menu placement helper. |
| 2026-07-13 | direct slice 59c06f8 | `description/13` | Project color now renders as a full-width bottom underline strip (`.project-color-underline`), not a title-box background fill. |
| 2026-07-13 | ci a189705, af4a8d2 | `description/12` | Release model reworked: `dev` push builds a rolling unversioned `rc` pre-release (`release-candidate.yml`); `master` push publishes the stable tagged release (`release.yml`, fail-loud on non-clean version, deletes the `rc`). Website stays with `pages.yml`. |
