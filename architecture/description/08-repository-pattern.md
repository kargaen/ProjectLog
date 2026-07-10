## Repository Pattern

Repositories abstract persistence entirely. Controllers never know the storage format.

```rust
// src-tauri/src/models/repository_traits/log_repository.rs
pub trait LogRepository {
    fn append(&self, entry: LogEntry) -> Result<(), String>;
    fn list_for_day(&self, day: NaiveDate) -> Result<Vec<LogEntry>, String>;
    fn list_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<LogEntry>, String>;
}
```

Swapping the storage format (flat file → structured format) requires changing only the concrete repository in `repositories/`, not the controller.
