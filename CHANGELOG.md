## Unreleased

- Full timesheet previews and Excel exports now open on the current week, falling back to the newest recorded week when needed. (EPIC-010)
- Project list colors now render as left-edge accents, groups render as collapsible boxes, manual drag stays within a project's current group, and empty group names remain reusable from the context menu. (EPIC-008)
- Architecture rewrite: domain-based folder structure, split bridge services, MVC layer separation
- Rounding is now applied on export (Excel), not just on the preview display
- Fixed total row double-counting comment hours when rounding is enabled
- Fixed taskbar button not hiding when always-on-top is enabled
- Fixed Yesterday + Today showing empty yesterday column
- Fixed Full timesheet showing stale content when the preview window was already open
- Projects can be tagged with a color that fills the row, and grouped so named groups list above ungrouped projects; the right-click project menu no longer gets clipped at the window edge
- Release pipeline: a release candidate can no longer be built for a version that has already shipped — the build fails with instructions instead; bumping to the next patch/minor/rc version is now a single command (EPIC-009)
