## Self-Documenting Folder Rule

A folder should answer one of these questions on sight:

| Question                        | Folder                        |
| ------------------------------- | ----------------------------- |
| Is this domain shape?           | `models/`                     |
| Is this business behavior?      | `controllers/`                |
| Is this rendering?              | `views/`                      |
| Is this transport?              | `services/bridge/` / `commands/` |
| Is this persistence detail?     | `repositories/`               |
| Is this OS-level glue?          | `infrastructure/`             |
| Is this shared reactive state?  | `stores/`                     |

If a file cannot be placed confidently, that usually means the responsibility is still unclear.
