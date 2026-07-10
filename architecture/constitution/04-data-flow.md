## Data Flow

Strict one-directional flow. No layer may import from a layer above it.

### Frontend flow

```
Svelte View
    ↓  user action (callback)
Frontend Controller
    ↓  calls
Bridge Service
    ↓  invoke()
Tauri Command
    ↓  delegates immediately
Native Controller
    ↓  reads / writes
Repository / Service
    ↓  side effects
Files / OS APIs / Window / Tray / Updater
```

### State returning to UI

```
Repository / Native Service
    ↓  returns DTO
Native Controller
    ↓  through Tauri command return value or event
Bridge Service
    ↓  deserializes and returns typed result
Frontend Controller
    ↓  writes to Svelte store (only on success)
Svelte View
    ↓  reactive derivation from store
```

### Ownership chain

```
Repository (reads/writes data)
    → Controller (orchestrates, enforces rules)
    → Store (broadcasts committed state)
    → View (renders)
```

### Tray flow (no frontend involved)

```
User clicks tray item
    ↓
Tray event handler (infrastructure/tray.rs)
    ↓  delegates immediately
Native Controller
    ↓  reads / writes
Repository / Service
    ↓  optionally emits Tauri window event if QuickPanel is open
Frontend Controller re-syncs store
```

### Hard rules

- Views never import from bridge services, repositories, or stores
- Views never call `invoke` directly
- Frontend controllers never import Svelte view components
- Bridge services never contain business rules
- Tauri commands never contain domain logic beyond transport mapping
- Native repositories never depend on window, tray, or UI concerns
- Tray handlers delegate to controllers immediately — they are not controllers themselves
- Stores are only written to after the native side effect succeeds
- Transient state (loading, draft values, dialog open/closed) lives in controller `$state`, not in stores
- Shared types live in `models/types/`, not inside views or controllers
