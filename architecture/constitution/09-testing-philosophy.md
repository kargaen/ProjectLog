## Testing Philosophy

The MVC split makes each layer independently testable with small, focused mocks.

| Layer                     | What to test                                              | Mock boundary                       |
| ------------------------- | --------------------------------------------------------- | ----------------------------------- |
| Frontend `models/schemas` | Payload parsing and invalid-state rejection               | None needed                         |
| Frontend controllers      | Async state, command sequencing, error handling           | Mock bridge services                |
| Frontend views            | Render behavior, callbacks, keyboard and input flows      | Props only — no stores or bridges   |
| Bridge services           | Command normalization and return type mapping             | Mock `invoke`                       |
| Native controllers        | Domain workflows and rule enforcement                     | Mock repositories and services      |
| Native repositories       | File/storage behavior and data mapping                    | Mock filesystem / infrastructure    |
| `timesheet_service`       | Hour aggregation, rounding, comment grouping, range logic | Mock repositories or clock          |
| `migration_service`       | Legacy format detection, upgrade correctness              | Mock filesystem                     |
| End-to-end                | Real desktop behavior across both surfaces                | Playwright                          |

**Critical regression paths:**

1. Selecting a project in QuickPanel starts a session and updates tray state
2. Selecting a project in the tray starts a session and updates QuickPanel if open
3. Saving a comment attaches it to the correct session block in the log
4. Timesheet preview shows correct per-project daily hour totals
5. Rounding toggle changes displayed totals without mutating source log
6. Exporting to Excel produces a valid file matching the preview
7. Settings (opacity, always-on-top, compact mode) survive app restart
8. Window bounds are restored correctly on reopen
