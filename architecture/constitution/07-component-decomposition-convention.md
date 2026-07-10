## Component Decomposition Convention

Every non-trivial view component splits into up to three files in its own folder:

```
ProjectList/
├── ProjectList.view.svelte   ← markup and prop destructuring only
├── ProjectList.styles.ts     ← all style objects
├── ProjectList.hooks.ts      ← formatting, local state, derived values
└── index.ts                  ← re-exports the view component
```

**`*.view.svelte`** — contains only markup and prop destructuring. No conditional logic beyond template branching. No format calls. No local `$state`. Calls to handlers are passed in as callbacks via `$props()`.

**`*.styles.ts`** — exports plain style constant objects. No imports from the MVC layers. No reactive code.

**`*.hooks.ts`** — exports a `create{Name}Hooks` function that takes props and returns derived values and formatting helpers. Uses `$derived` for reactive derivations. No side effects. No store writes.

Simple presentational components (e.g. `SortTabs`, `RoundingToggle`) that need no formatting or local state may use a single file without a folder.
