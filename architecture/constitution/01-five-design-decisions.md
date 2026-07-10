## Five Design Decisions That Keep This Codebase Stable

These five decisions are the load-bearing walls of the architecture. Every naming convention, folder rule, and layer boundary below exists to enforce them. When a bug cascades, it is almost always because one of these was violated.

**1. The import graph flows in one direction only.**
Views import from controllers. Controllers import from bridge services and stores. Bridge services import from models. Nothing imports from a layer above it. This means you can always locate a bug by asking which layer owns the broken invariant — there is no "could be anywhere" debugging.

**2. Repositories own every storage decision.**
Nothing above the repository layer knows whether data is in a flat file, JSON, or a different format. Controllers never call filesystem APIs or `invoke` storage commands directly. Swapping a storage backend or format requires changing exactly one file.

**3. Stores are write-through, not write-ahead.**
Controllers write to a Svelte store only after the native side effect (repository write via Tauri command) succeeds. The store is always a mirror of committed state, never an optimistic prediction. Transient UI state (loading flags, draft values, open dialogs) lives in the controller's local `$state`, not in shared stores.

**4. The component decomposition convention enforces zero logic in markup.**
Every non-trivial component splits into three files: `Name.view.svelte` for markup only, `Name.styles.ts` for all StyleSheet-equivalent style objects, and `Name.hooks.ts` for any formatting, local state, or derived values. The `.view.svelte` file may only contain JSX-equivalent markup and prop destructuring. No conditionals, no format calls, no event logic beyond passing callbacks through.

**5. Two surfaces share one domain. Neither owns state.**
The QuickPanel window and the system tray are both surfaces over the same native domain state. Neither surface caches, derives, or manages its own version of the truth. Both call into the same native controllers. When either surface acts, the result is a fresh state snapshot returned from the native layer, which both surfaces then reflect.
