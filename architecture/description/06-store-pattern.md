## Store Pattern

Stores hold committed domain state. They are written by controllers and read by screens and view hooks. View components never read from stores directly — they receive store-derived state as props from screens.

```ts
// src/stores/projectStore.ts
import { writable, derived } from 'svelte/store';
import type { ProjectState } from '../models/types';

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectState | null>(null);

  return {
    subscribe,
    apply: (state: ProjectState) => set(state),
    reset: () => set(null),
  };
}

export const projectStore = createProjectStore();
```

**Store rules:**
- One store per domain slice
- Stores expose a minimal mutation surface (`apply`, `reset`) — not a generic `set`
- Controllers call `apply` only after the native side effect succeeds
- Transient state (loading flags, draft values, dialog states) lives in controller `$state`, not in stores
- Derived values (e.g. sorted project list, active project name) use Svelte `derived()` inside the store or in component hooks — not recomputed inline in views
