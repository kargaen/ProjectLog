## Architectural Goals

The architecture must serve the actual purpose of the product:

1. **Fast project switching** — selecting what you are working on takes seconds, from either surface.
2. **Reliable daily hour logging** — users must be able to reconstruct hours spent per project each day with confidence.
3. **Dual-surface consistency** — QuickPanel and the tray operate on the same domain state; neither surface sees data the other cannot.
4. **Local-first trust** — project names, comments, timestamps, and settings remain on the user's machine with no network dependency.
5. **Safe refactoring** — the codebase must support aggressive reorganization without blurring responsibilities.
6. **Clear expansion path** — reminders, timesheet rounding, export formats, tray flows, and reporting must fit the architecture cleanly.
