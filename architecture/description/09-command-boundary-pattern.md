## Command Boundary Pattern

Tauri commands are transport adapters only. If a command contains business logic, that logic belongs in a controller.

```rust
// src-tauri/src/commands/session_commands.rs
#[tauri::command]
pub fn start_tracking(project: String, state: State<AppState>) -> Result<ProjectStateDto, String> {
    state.session_controller.start_tracking(&project)
}
```
