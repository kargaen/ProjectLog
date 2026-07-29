## Stack

| Concern             | Choice                   | Notes                                                                                 |
| ------------------- | ------------------------ | ------------------------------------------------------------------------------------- |
| Desktop shell       | Tauri v2                 | Lightweight desktop shell with native windowing, tray, autostart, updater support    |
| Native dialogs      | Tauri dialog plugin      | QuickPanel and tray confirmations use OS-native dialogs instead of browser prompts   |
| Frontend UI         | Svelte 5                 | Reactive UI with rune-based reactivity and low boilerplate                            |
| Frontend language   | TypeScript               | Strict typing for models, controller APIs, and bridge contracts                      |
| Build tool          | Vite                     | Fast frontend build and local development                                             |
| Native language     | Rust                     | Reliable filesystem access, structured domain logic, and desktop integration          |
| Testing             | Playwright + Cargo tests | UI regression coverage plus Rust unit/integration tests                               |
| Packaging/updates   | Tauri updater            | App updates remain separate from domain logic                                         |
| Persistence         | Local files only         | Local-first storage for projects, logs, settings, and generated timesheet data        |
