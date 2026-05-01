<script lang="ts">
  let {
    isCompactLayout,
    onToggleQuickPanelMode,
  }: {
    isCompactLayout: boolean;
    onToggleQuickPanelMode: () => void | Promise<void>;
  } = $props();
</script>

<section class="panel footer compact-mode-footer">
  <div class="toggle-row">
    <span>Compact mode</span>
    <button
      aria-label="Compact mode"
      class:toggle-on={isCompactLayout}
      class="toggle"
      onclick={onToggleQuickPanelMode}
    >
      <span></span>
    </button>
  </div>
</section>
