## Tray Handler Pattern

Tray handlers are event adapters. They mirror the command boundary pattern: receive an event, delegate to a controller immediately, optionally sync the QuickPanel window.

```rust
// src-tauri/src/infrastructure/tray.rs
SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
    project if known_projects.contains(project) => {
        app.state::<AppState>().session_controller.start_tracking(project).ok();
        sync_quickpanel_if_open(&app);
    }
    "open_quickpanel" => {
        app.state::<AppState>().shell_controller.show_quickpanel();
    }
    _ => {}
}
```
