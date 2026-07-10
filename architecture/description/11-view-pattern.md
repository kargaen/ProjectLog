## View Pattern

Views receive already-prepared state and emit intent via callbacks. A view knows what to show, but never what the business rules are.

```svelte
<!-- src/views/components/projects/ProjectRow.view.svelte -->
<script lang="ts">
  let { name, isActive, onSelect, onRemove }: {
    name: string;
    isActive: boolean;
    onSelect: (name: string) => void;
    onRemove: (name: string) => void;
  } = $props();
</script>

<div class="row" class:active={isActive}>
  <button onclick={() => onSelect(name)}>{name}</button>
  <button onclick={() => onRemove(name)}>×</button>
</div>
```
