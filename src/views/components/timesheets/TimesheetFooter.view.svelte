<script lang="ts">
  let {
    roundingEnabled,
    onToggleRounding,
    onClose,
    onExport,
  }: {
    roundingEnabled: boolean;
    onToggleRounding: () => void | Promise<void>;
    onClose: () => void | Promise<void>;
    onExport: () => void | Promise<void>;
  } = $props();
</script>

<section class="timesheet-window-footer">
  <div class="toggle-row">
    <span>Round to 0.5h</span>
    <button
      aria-label="Round to 0.5h"
      class:toggle-on={roundingEnabled}
      class="toggle"
      onclick={onToggleRounding}
    >
      <span></span>
    </button>
  </div>
  <div class="dialog-buttons">
    <button onclick={onClose}>Close</button>
    <button class="primary" onclick={onExport}>Export to Excel</button>
  </div>
</section>
