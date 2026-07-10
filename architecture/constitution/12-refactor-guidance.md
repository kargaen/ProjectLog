## Refactor Guidance

During any refactor pass, one file at a time:

- Do not move business logic into Svelte views
- Do not let Tauri commands become controller substitutes
- Do not let tray handlers contain domain logic
- Do not hide domain rules inside utility files
- Do not let desktop-shell code leak into project/session/timesheet domains
- Do not optimize for current file layout over long-term clarity

When a refactor pass breaks downstream files intentionally, state that clearly. The app is allowed to be broken between layer passes — the agent working the refactor must not leave backwards-compatibility shims behind to avoid that breakage. Make the clean change and document what must follow.
