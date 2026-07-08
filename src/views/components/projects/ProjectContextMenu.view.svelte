<script lang="ts">
  const COLOR_PRESETS = [
    "#e05d44",
    "#e8974f",
    "#d9b13c",
    "#267a62",
    "#3f8fd1",
    "#7c6fd1",
    "#d1618f",
    "#8a8f99",
  ];

  let {
    x,
    y,
    currentColor,
    currentGroup,
    knownGroupNames,
    onPickColor,
    onPickGroup,
    onRequestNewGroup,
    onClose,
  }: {
    x: number;
    y: number;
    currentColor: string | null;
    currentGroup: string | null;
    knownGroupNames: string[];
    onPickColor: (color: string | null) => void;
    onPickGroup: (group: string | null) => void;
    onRequestNewGroup: () => void;
    onClose: () => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="context-menu-backdrop"
  onclick={onClose}
  oncontextmenu={(event) => {
    event.preventDefault();
    onClose();
  }}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="project-context-menu"
    style="left: {x}px; top: {y}px;"
    onclick={(event) => event.stopPropagation()}
  >
    <div class="context-menu-section-label">Color</div>
    <div class="context-menu-swatches">
      <button
        class="swatch swatch-clear"
        class:selected={!currentColor}
        title="Clear color"
        onclick={() => onPickColor(null)}
      >
        x
      </button>
      {#each COLOR_PRESETS as color}
        <button
          class="swatch"
          class:selected={currentColor === color}
          style="background: {color};"
          title={color}
          onclick={() => onPickColor(color)}
        ></button>
      {/each}
    </div>
    <div class="context-menu-section-label">Group</div>
    <div class="context-menu-groups">
      <button
        class="context-menu-item"
        class:selected={!currentGroup}
        onclick={() => onPickGroup(null)}
      >
        Ungrouped
      </button>
      {#each knownGroupNames as group}
        <button
          class="context-menu-item"
          class:selected={currentGroup === group}
          onclick={() => onPickGroup(group)}
        >
          {group}
        </button>
      {/each}
      <button class="context-menu-item ghost" onclick={onRequestNewGroup}>
        + New group...
      </button>
    </div>
  </div>
</div>
