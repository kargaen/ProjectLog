## Frontend Controller Pattern

Controllers are the only layer screen components interact with. They own async state, call bridge services, normalize data, write to stores, and expose a stable action surface. They never render markup.

```ts
// src/controllers/projects/createProjectListController.ts
export function createProjectListController() {
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function selectProject(name: string) {
    loading = true;
    error = null;
    try {
      const nextState = await projectBridge.selectProject(name);
      projectStore.apply(nextState); // write-through: only on success
    } catch {
      error = "Could not select project.";
    } finally {
      loading = false;
    }
  }

  return {
    selectProject,
    get loading() { return loading; },
    get error()   { return error; },
  };
}
```

**What a controller owns:**
- Local `loading` and `error` `$state`
- All business rules for its domain (validation, sequencing, error handling)
- Calls to bridge services
- Store writes (only after side effects succeed)
- The action surface returned to screens

**What a controller never does:**
- Returns or imports JSX/Svelte markup
- Imports from other controllers
- Calls `invoke` directly (delegates to bridge)
- Writes to a store before the native side effect succeeds
