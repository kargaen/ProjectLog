## Two Surfaces, One Domain

ProjectLog exposes two interaction surfaces over the same underlying native domain.

### QuickPanel

A floating Tauri window with full access to all features:

- Project list with sort modes (Manual, A-Z, Recent) and drag-to-reorder
- Comment field that attaches context to the active session
- Add project and Quick track (ad-hoc) inputs
- Timesheet actions: Full timesheet, Yesterday + today, Open log file, Reset timesheet
- Settings: Always on top, Open on start, Opacity, Compact mode

### System Tray

A native system tray menu that mirrors core QuickPanel actions without requiring the window to be open:

- Project list for direct activation, with named groups shown as native submenus when grouping is enabled
- Set comment
- Add / Quick add / Remove project
- Generate timesheet (range submenu), Open log file, Reset timesheet, Reset projects
- Open diagnostic log, Feedback, About, Exit

**Architectural implication:** neither surface owns the domain. Both call into the same native controllers through Tauri commands or tray event handlers. State is owned by the native layer and returned to whichever surface is active.
