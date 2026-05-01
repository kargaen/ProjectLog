<script lang="ts">
  let {
    title,
    years,
    generatedStatus,
    refreshing,
    onStartDrag,
    onRefreshNow,
  }: {
    title: string;
    years: string[];
    generatedStatus: string;
    refreshing: boolean;
    onStartDrag: () => void | Promise<void>;
    onRefreshNow: () => void | Promise<void>;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="timesheet-window-header" onmousedown={onStartDrag}>
  <div>
    <h1>{title}</h1>
    {#if years.length > 1}
      <p class="timesheet-year-hint">Includes weeks from {years.join(", ")}</p>
    {/if}
  </div>

  <div class="timesheet-status-shell">
    <div class="timesheet-generated-status">{generatedStatus}</div>
    <button class="ghost small" onclick={onRefreshNow} disabled={refreshing}>
      {refreshing ? "Updating..." : "Update now"}
    </button>
  </div>
</header>
