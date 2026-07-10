## Domain Boundaries

ProjectLog is organized around six stable business domains.

### 1. Project Domain

- Permanent projects and ad-hoc / quick projects
- Manual ordering and drag-to-reorder
- Sort modes: Manual, A-Z, Recent
- Recent usage metadata for sort and display (Recent sort tracks multiple projects, not just the last one)
- Name uniqueness enforcement

The log itself is agnostic about project names — it accepts any string. The distinction between a permanent project and an ad-hoc entry is purely a UI concern. Both produce identical log entries. Only the UI decides whether a name is surfaced in the remembered project list.

### 2. Session / Log Domain

- Active project state (which project is currently being tracked)
- Single-timestamp transitions: switching to a project (or to blank) writes one timestamp that simultaneously closes the previous session and opens the new one — there is no separate stop/start pair
- Comment attachment to the active session block
- Append-only log entries that preserve full history
- Reconstructing daily work history from log data
- On app open, active project is always blank — the app never assumes the user is still working

**Log format:** each line begins with a newline and a timestamp followed by a tab. The line has no trailing newline — the cursor sits ready for either a project name continuation (new line + timestamp + tab + name) or an inline comment (tab + comment text). This means a comment is structurally part of the same line as its session-start timestamp, making misattachment impossible by format rather than by application logic.

### 3. Timesheet Domain

- Aggregating log entries into per-project daily hour totals
- Producing a weekly table (Mon–Sun columns, project rows, comment sub-rows)
- Range selection: full history, this week, yesterday + today
- Rounding rules (e.g. round to 0.5h) with deliberate anti-inflation logic to prevent rounding from artificially increasing total reported hours
- Preview generation for the Timesheet window
- Export serialization to Excel

**Intentional staleness:** the timesheet preview is a snapshot generated on demand, not a live view. It does not update automatically while open. The user refreshes it explicitly by clicking the refresh button or by closing and reopening the window. This is by design — a stable preview the user can review and export without it changing under them.

This domain is core to the product mission. The timesheet output is the primary deliverable users rely on for daily hour reporting.

### 4. Settings Domain

- Always-on-top window behavior
- Open QuickPanel on system start
- QuickPanel opacity (0–100%)
- Compact mode (reduced window height)
- Window position and size persistence
- Timesheet rounding rules

### 5. Desktop Shell Domain

- System tray menu construction and state
- Tray menu item enabling/disabling based on active session
- Window visibility, focus, and placement
- Autostart integration
- Updater workflows
- Desktop lifecycle events (minimize to tray, restore, close)

### 6. Diagnostics Domain

- Structured diagnostic log for troubleshooting
- App health inspection (storage paths, file integrity, version)
- Debug-safe support output
- Migration and format upgrade visibility
