## Native Controller Pattern

Native controllers own application rules. Tauri commands and tray handlers forward to them immediately — neither is a substitute for a controller.

```rust
// src-tauri/src/controllers/session_controller.rs
pub struct SessionController<L: LogRepository, P: ProjectRepository> {
    log_repo: L,
    project_repo: P,
}

impl<L: LogRepository, P: ProjectRepository> SessionController<L, P> {
    pub fn start_tracking(&self, project: &str) -> Result<ProjectStateDto, String> {
        let entry = LogEntry::start(project, Utc::now());
        self.log_repo.append(entry)?;
        self.project_repo.set_active(project)?;
        self.project_repo.record_recent_usage(project)?;
        self.project_repo.get_state()
    }
}
```
