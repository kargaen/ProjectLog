<script lang="ts">
  import type { TimesheetDisplayRow } from "../../../lib/timesheet-preview";

  let {
    columns,
    rows,
    hoveredRowIndex,
    hoveredColumnIndex,
    onClearCrosshair,
    onSetHoveredCell,
    onSelectSheet,
    sheets,
    selectedSheetIndex,
    formatHours,
  }: {
    columns: string[];
    rows: TimesheetDisplayRow[];
    hoveredRowIndex: number | null;
    hoveredColumnIndex: number | null;
    onClearCrosshair: () => void;
    onSetHoveredCell: (rowIndex: number, columnIndex: number) => void;
    onSelectSheet: (index: number) => void;
    sheets: string[];
    selectedSheetIndex: number;
    formatHours: (value: number) => string;
  } = $props();
</script>

<section class="timesheet-preview-panel">
  {#if sheets.length > 1}
    <div class="sheet-tabs">
      {#each sheets as sheet, index}
        <button
          class:sort-active={selectedSheetIndex === index}
          onclick={() => onSelectSheet(index)}
        >
          {sheet}
        </button>
      {/each}
    </div>
  {/if}

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="timesheet-table-wrap" onmouseleave={onClearCrosshair}>
    <table class="timesheet-table">
      <thead>
        <tr>
          <th class:crosshair-column={hoveredColumnIndex === 0}>Project</th>
          {#each columns as column, columnIndex}
            <th class:crosshair-column={hoveredColumnIndex === columnIndex + 1}>
              {column}
            </th>
          {/each}
          <th class:crosshair-column={hoveredColumnIndex === columns.length + 1}>
            Total
          </th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row, rowIndex}
          <tr
            class:comment-row={row.is_comment}
            class:total-row={row.is_total}
            class:banded-row={row.band_index >= 0 && row.band_index % 2 === 1}
            class:crosshair-row={hoveredRowIndex === rowIndex}
          >
            <td
              class:crosshair-column={hoveredColumnIndex === 0}
              onmouseenter={() => onSetHoveredCell(rowIndex, 0)}
            >
              {row.label}
            </td>
            {#each row.values as value, columnIndex}
              <td
                class:crosshair-column={hoveredColumnIndex === columnIndex + 1}
                onmouseenter={() => onSetHoveredCell(rowIndex, columnIndex + 1)}
              >
                {formatHours(value)}
              </td>
            {/each}
            <td
              class:crosshair-column={hoveredColumnIndex === columns.length + 1}
              onmouseenter={() => onSetHoveredCell(rowIndex, columns.length + 1)}
            >
              {formatHours(row.total)}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
