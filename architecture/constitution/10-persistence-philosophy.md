## Persistence Philosophy

1. User project data, comments, timestamps, and settings are stored locally only — no cloud, no auth.
2. The log is append-only. Timesheet generation reads from it; nothing writes to it except session transitions.
3. Persistence format is an implementation detail hidden behind repository traits.
4. The app must tolerate missing, partial, or legacy files through startup migration services.
5. Timesheet generation must be fully reproducible from persisted log data at any time.
6. Export logic must never mutate source log history.
