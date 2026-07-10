## Architecture Philosophy

ProjectLog uses a strict MVC split adapted to **Svelte 5 on the frontend** and **Tauri + Rust on the native side**:

| Layer          | Where it lives                                      | Responsibility                                                                                           |
| -------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Model**      | `src/models/` and `src-tauri/src/models/`           | Shape of data — TypeScript/Rust domain types, validation schemas, persistence abstractions, repository contracts |
| **View**       | `src/views/`                                        | Pure presentation — Svelte components that receive state and emit callbacks, with no business decisions  |
| **Controller** | `src/controllers/` and `src-tauri/src/controllers/` | Business logic — orchestrates models, drives UI state, coordinates native commands, enforces workflows   |

Because ProjectLog is a desktop app with a frontend/native split, MVC applies on **both sides**:

- The **frontend MVC** controls rendering, interaction flow, and view state.
- The **native MVC** controls persistence, file I/O, timesheet generation, tray behavior, and OS integration.

The boundary between them is a thin, typed transport layer:

- **Frontend controllers** call typed command clients in `src/services/bridge/`
- **Views** never call Tauri commands directly
- **Rust commands** are transport endpoints only — they delegate immediately to native controllers
