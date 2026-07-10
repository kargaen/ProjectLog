## Import Graph Rule

Each layer may only import from layers below it. Violations are bugs.

### Frontend

```
┌──────────────────────────────────────────┐
│  Screens                                 │  imports controllers, stores, view components
├──────────────────────────────────────────┤
│  View components                         │  imports ui primitives, utils — nothing else
├──────────────────────────────────────────┤
│  Controllers                             │  imports bridge services, stores, models/types
├──────────────────────────────────────────┤
│  Bridge services / Stores                │  imports models/types only
├──────────────────────────────────────────┤
│  Models / schemas / utils                │  no MVC imports
└──────────────────────────────────────────┘
```

Screens are the only layer permitted to read from stores and instantiate controllers. View components never read from stores — they receive state as props.

### Native

```
┌──────────────────────────────────────────┐
│  Commands / Tray handlers                │  imports controllers and state only
├──────────────────────────────────────────┤
│  Controllers                             │  imports repositories, services, models, state
├──────────────────────────────────────────┤
│  Repositories / Services                 │  imports models, infrastructure, utils
├──────────────────────────────────────────┤
│  Models / DTOs / traits                  │  no UI concerns, no OS concerns
└──────────────────────────────────────────┘
```
