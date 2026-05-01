<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { createLogger } from "../logger";
  import {
    buildTimesheetDisplayRows,
    formatGeneratedAge,
    formatGeneratedTimestamp,
    formatPreviewHours,
  } from "../timesheet-preview";
  import type {
    TimesheetFormat,
    TimesheetPreview,
    TimesheetPreviewBootstrap,
    TimesheetPreviewRequest,
    TimesheetRange,
  } from "../types";

  const log = createLogger("timesheet-preview");

  let timesheetPreview = $state<TimesheetPreview | null>(null);
  let timesheetPreviewRange = $state<TimesheetRange>("all");
  let timesheetPreviewFormat = $state<TimesheetFormat>("full");
  let timesheetPreviewSheetIndex = $state(0);
  let timesheetRoundingEnabled = $state(false);
  let loading = $state(true);
  let refreshing = $state(false);
  let hoveredRowIndex = $state<number | null>(null);
  let hoveredColumnIndex = $state<number | null>(null);
  let relativeTimeNow = $state(Date.now());

  const currentWindow = getCurrentWindow();
  function formatHours(value: number) {
    return formatPreviewHours(value, timesheetRoundingEnabled);
  }

  async function startWindowDrag() {
    await currentWindow.startDragging().catch(() => {});
  }

  function clearCrosshair() {
    hoveredRowIndex = null;
    hoveredColumnIndex = null;
  }

  function updateSheetIndex(nextPreview: TimesheetPreview, format: TimesheetFormat) {
    const currentSheetName = displayedTimesheetSheet?.name;
    if (format !== "full") {
      timesheetPreviewSheetIndex = 0;
      return;
    }

    const matchingIndex = nextPreview.sheets.findIndex((sheet) => sheet.name === currentSheetName);
    timesheetPreviewSheetIndex =
      matchingIndex >= 0 ? matchingIndex : Math.max(nextPreview.sheets.length - 1, 0);
  }

  async function loadPreview(
    range: TimesheetRange,
    format: TimesheetFormat,
    options?: { preservePreview?: boolean }
  ) {
    const preservePreview = options?.preservePreview ?? false;
    timesheetPreviewRange = range;
    timesheetPreviewFormat = format;

    if (!preservePreview) {
      timesheetPreview = null;
      loading = true;
    } else {
      refreshing = true;
    }

    try {
      const preview = await invoke<TimesheetPreview>("preview_timesheet", { range, format });
      timesheetPreview = preview;
      updateSheetIndex(preview, format);
      clearCrosshair();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      log.warn("loadPreview failed", { range, format, message });
      alert(message);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function refreshNow() {
    await loadPreview(timesheetPreviewRange, timesheetPreviewFormat, {
      preservePreview: Boolean(timesheetPreview),
    });
  }

  async function closeTimesheetPreviewWindow() {
    await invoke("hide_timesheet_preview_window").catch(() => {});
  }

  async function toggleTimesheetRounding() {
    const next = !timesheetRoundingEnabled;
    timesheetRoundingEnabled = next;
    await invoke("set_timesheet_rounding_enabled", { enabled: next });
  }

  async function exportTimesheet() {
    try {
      await invoke("generate_timesheet_export", {
        range: timesheetPreviewRange,
        format: timesheetPreviewFormat,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      log.warn("exportTimesheet failed", { message });
      alert(message);
    }
  }

  const displayedTimesheetSheet = $derived(
    timesheetPreview ? timesheetPreview.sheets[timesheetPreviewSheetIndex] : null
  );

  const timesheetPreviewYears = $derived.by(() => {
    if (!timesheetPreview) return [];
    const years = new Set(
      timesheetPreview.sheets
        .map((sheet) => sheet.name.split("-")[0])
        .filter((value) => /^\d{4}$/.test(value))
    );
    return [...years].sort();
  });

  const displayedTimesheetRows = $derived(
    buildTimesheetDisplayRows(displayedTimesheetSheet, timesheetRoundingEnabled)
  );

  const generatedStatus = $derived.by(() => {
    if (!timesheetPreview) return "";
    return `Generated at ${formatGeneratedTimestamp(timesheetPreview.generated_at_epoch_ms)}, ${formatGeneratedAge(timesheetPreview.generated_at_epoch_ms, relativeTimeNow)}`;
  });

  onMount(() => {
    log.info("mounted");
    const timer = setInterval(() => {
      relativeTimeNow = Date.now();
    }, 1000);

    let disposed = false;
    let cleanup = () => {
      clearInterval(timer);
    };

    void (async () => {
      const unlistenTimesheetPreview = currentWindow.listen<TimesheetPreviewRequest>(
        "show-timesheet-preview",
        (event) => {
          loadPreview(event.payload.range, event.payload.format, {
            preservePreview: Boolean(timesheetPreview),
          });
        }
      );

      cleanup = () => {
        clearInterval(timer);
        unlistenTimesheetPreview.then((fn) => fn());
      };

      try {
        const bootstrap = await invoke<TimesheetPreviewBootstrap>("get_timesheet_preview_bootstrap");
        timesheetRoundingEnabled = bootstrap.rounding_enabled;
        if (bootstrap.request) {
          await loadPreview(bootstrap.request.range, bootstrap.request.format);
        } else {
          loading = false;
        }
      } catch (error) {
        loading = false;
        const message = error instanceof Error ? error.message : String(error);
        log.warn("bootstrap failed", { message });
        alert(message);
      }

      if (disposed) {
        cleanup();
      }
    })();

    return () => {
      disposed = true;
      cleanup();
    };
  });
</script>

<main class="timesheet-window">
  {#if timesheetPreview && displayedTimesheetSheet}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="timesheet-window-header" onmousedown={startWindowDrag}>
      <div>
        <h1>{timesheetPreview.title}</h1>
        {#if timesheetPreviewYears.length > 1}
          <p class="timesheet-year-hint">
            Includes weeks from {timesheetPreviewYears.join(", ")}
          </p>
        {/if}
      </div>

      <div class="timesheet-status-shell">
        <div class="timesheet-generated-status">{generatedStatus}</div>
        <button class="ghost small" onclick={refreshNow} disabled={refreshing}>
          {refreshing ? "Updating..." : "Update now"}
        </button>
      </div>
    </header>

    <section class="timesheet-preview-panel">
      {#if timesheetPreview.sheets.length > 1}
        <div class="sheet-tabs">
          {#each timesheetPreview.sheets as sheet, index}
            <button
              class:sort-active={timesheetPreviewSheetIndex === index}
              onclick={() => {
                timesheetPreviewSheetIndex = index;
                clearCrosshair();
              }}
            >{sheet.name}</button>
          {/each}
        </div>
      {/if}

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="timesheet-table-wrap" onmouseleave={clearCrosshair}>
        <table class="timesheet-table">
          <thead>
            <tr>
              <th class:crosshair-column={hoveredColumnIndex === 0}>Project</th>
              {#each displayedTimesheetSheet.columns as column, columnIndex}
                <th class:crosshair-column={hoveredColumnIndex === columnIndex + 1}>{column}</th>
              {/each}
              <th class:crosshair-column={hoveredColumnIndex === displayedTimesheetSheet.columns.length + 1}>
                Total
              </th>
            </tr>
          </thead>
          <tbody>
            {#each displayedTimesheetRows as row, rowIndex}
              <tr
                class:comment-row={row.is_comment}
                class:total-row={row.is_total}
                class:banded-row={row.band_index >= 0 && row.band_index % 2 === 1}
                class:crosshair-row={hoveredRowIndex === rowIndex}
              >
                <td
                  class:crosshair-column={hoveredColumnIndex === 0}
                  onmouseenter={() => {
                    hoveredRowIndex = rowIndex;
                    hoveredColumnIndex = 0;
                  }}
                >{row.label}</td>
                {#each row.values as value, columnIndex}
                  <td
                    class:crosshair-column={hoveredColumnIndex === columnIndex + 1}
                    onmouseenter={() => {
                      hoveredRowIndex = rowIndex;
                      hoveredColumnIndex = columnIndex + 1;
                    }}
                  >{formatHours(value)}</td>
                {/each}
                <td
                  class:crosshair-column={hoveredColumnIndex === displayedTimesheetSheet.columns.length + 1}
                  onmouseenter={() => {
                    hoveredRowIndex = rowIndex;
                    hoveredColumnIndex = displayedTimesheetSheet.columns.length + 1;
                  }}
                >{formatHours(row.total)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <section class="timesheet-window-footer">
      <div class="toggle-row">
        <span>Round to 0.5h</span>
        <button
          aria-label="Round to 0.5h"
          class:toggle-on={timesheetRoundingEnabled}
          class="toggle"
          onclick={toggleTimesheetRounding}
        ><span></span></button>
      </div>
      <div class="dialog-buttons">
        <button onclick={closeTimesheetPreviewWindow}>Close</button>
        <button class="primary" onclick={exportTimesheet}>Export to Excel</button>
      </div>
    </section>
  {:else if loading}
    <section class="timesheet-preview-panel timesheet-preview-loading">
      <h1>Loading timesheet...</h1>
      <p>ProjectLog is preparing the preview window.</p>
    </section>
  {:else}
    <section class="timesheet-preview-panel timesheet-preview-loading">
      <h1>No preview request</h1>
      <p>Open this window from QuickPanel to prepare a timesheet preview.</p>
    </section>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #eef1e8;
    color: #20241f;
  }

  main {
    min-height: 100vh;
  }

  h1,
  p {
    margin: 0;
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid #c9cebf;
    border-radius: 4px;
    background: #fff;
    color: #20241f;
    padding: 5px 8px;
    font-size: 11px;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: #f2f5ee;
  }

  button:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .primary {
    border-color: #267a62;
    background: #267a62;
    color: #fff;
  }

  .primary:hover:not(:disabled) {
    background: #216b56;
  }

  .ghost {
    color: #495047;
  }

  .small {
    padding: 4px 7px;
  }

  .timesheet-window {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    background: linear-gradient(180deg, #f5f7f1 0%, #eef1e8 100%);
    box-sizing: border-box;
  }

  .timesheet-window-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 4px 2px 0;
    cursor: default;
  }

  .timesheet-window-header h1 {
    color: #21352c;
    font-size: 20px;
  }

  .timesheet-window-header p {
    margin-top: 4px;
    color: #5f675d;
    font-size: 12px;
  }

  .timesheet-year-hint {
    color: #267a62;
    font-weight: 600;
  }

  .timesheet-status-shell {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: nowrap;
  }

  .timesheet-generated-status {
    display: inline-flex;
    align-items: center;
    padding: 7px 9px;
    border: 1px solid #ced8ae;
    border-radius: 6px;
    background: #e1e9bf;
    color: #405120;
    font-size: 11px;
    text-align: right;
    white-space: nowrap;
  }

  .timesheet-preview-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid #cfd5c6;
    border-radius: 8px;
    box-shadow: 0 14px 34px rgba(45, 55, 39, 0.1);
  }

  .timesheet-preview-loading {
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .timesheet-preview-loading h1 {
    color: #21352c;
    font-size: 18px;
  }

  .timesheet-preview-loading p {
    margin-top: 4px;
    color: #5f675d;
    font-size: 12px;
  }

  .sheet-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .sort-active {
    border-color: #267a62;
    background: #e9f6f1;
    color: #267a62;
  }

  .timesheet-table-wrap {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border: 1px solid #d8dccc;
    border-radius: 5px;
  }

  .timesheet-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
    background: #fff;
  }

  .timesheet-table th,
  .timesheet-table td {
    padding: 6px 8px;
    border-bottom: 1px solid #ecefe6;
    text-align: right;
    white-space: nowrap;
    transition:
      background-color 120ms ease,
      box-shadow 120ms ease;
  }

  .timesheet-table th:first-child,
  .timesheet-table td:first-child {
    text-align: left;
    min-width: 220px;
  }

  .timesheet-table thead th {
    position: sticky;
    top: 0;
    background: #dbe5c5;
    color: #304122;
    z-index: 1;
  }

  .banded-row td {
    background: #f3f7eb;
  }

  .comment-row td:first-child {
    color: #697064;
  }

  .comment-row td {
    color: #495047;
  }

  .total-row td {
    font-weight: 650;
    background: #dfe9cf;
    color: #26331c;
  }

  .crosshair-row td {
    background: #f8d9de;
  }

  .crosshair-row.banded-row td {
    background: #f3cfd7;
  }

  .crosshair-row.total-row td {
    background: #efc7cf;
  }

  .crosshair-column {
    box-shadow: inset 0 0 0 9999px rgba(244, 154, 169, 0.22);
  }

  .timesheet-window-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-top: 2px;
  }

  .toggle-row,
  .dialog-buttons {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .toggle-row span {
    flex: 1;
    font-size: 11px;
    color: #495047;
  }

  .toggle {
    width: 30px;
    height: 18px;
    border-radius: 10px;
    padding: 0;
    justify-content: flex-start;
    background: #d8dccc;
  }

  .toggle:hover:not(:disabled) {
    background: #d8dccc;
  }

  .toggle span {
    flex: none;
    width: 12px;
    height: 12px;
    margin-left: 2px;
    border-radius: 50%;
    background: #fff;
    transition: margin-left 120ms ease;
  }

  .toggle.toggle-on {
    background: #267a62;
  }

  .toggle.toggle-on:hover:not(:disabled) {
    background: #267a62;
  }

  .toggle.toggle-on span {
    margin-left: 14px;
  }

  @media (max-width: 640px) {
    .timesheet-window-header,
    .timesheet-window-footer {
      flex-direction: column;
      align-items: stretch;
    }

    .timesheet-status-shell {
      justify-content: flex-start;
    }

    .timesheet-generated-status {
      text-align: left;
    }

    .dialog-buttons {
      justify-content: flex-end;
    }
  }
</style>
