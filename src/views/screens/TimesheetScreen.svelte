<script lang="ts">
  import { onMount } from "svelte";

  import {
    createTimesheetPreviewController,
  } from "../../controllers/timesheets/createTimesheetPreviewController.svelte";
  import TimesheetFooter from "../components/timesheets/TimesheetFooter.view.svelte";
  import TimesheetHeader from "../components/timesheets/TimesheetHeader.view.svelte";
  import TimesheetStatePanel from "../components/timesheets/TimesheetStatePanel.view.svelte";
  import TimesheetTable from "../components/timesheets/TimesheetTable.view.svelte";
  import FontScaleIndicator from "../components/shared/FontScaleIndicator.view.svelte";
  import "./timesheet.css";

  const controller = createTimesheetPreviewController();
  const state = controller.state;
  const view = controller.view;

  onMount(() => controller.mount());
</script>

<main class="timesheet-screen">
  {#if state.timesheetPreview && view.displayedTimesheetSheet}
    <TimesheetHeader
      title={state.timesheetPreview.title}
      years={view.timesheetPreviewYears}
      generatedStatus={view.generatedStatus}
      refreshing={state.refreshing}
      onStartDrag={controller.startWindowDrag}
      onRefreshNow={controller.refreshNow}
    />

    <TimesheetTable
      columns={view.displayedTimesheetSheet.columns}
      rows={view.displayedTimesheetRows}
      hoveredRowIndex={state.hoveredRowIndex}
      hoveredColumnIndex={state.hoveredColumnIndex}
      onClearCrosshair={controller.clearCrosshair}
      onSetHoveredCell={controller.setHoveredCell}
      onSelectSheet={controller.selectSheet}
      sheets={state.timesheetPreview.sheets.map((sheet) => sheet.name)}
      selectedSheetIndex={state.timesheetPreviewSheetIndex}
      formatHours={controller.formatHours}
    />

    <TimesheetFooter
      roundingEnabled={state.timesheetRoundingEnabled}
      onToggleRounding={controller.toggleTimesheetRounding}
      onClose={controller.closeTimesheetPreviewWindow}
      onExport={controller.exportTimesheet}
    />
  {:else if state.loading}
    <TimesheetStatePanel
      title="Loading timesheet..."
      body="ProjectLog is preparing the preview window."
    />
  {:else}
    <TimesheetStatePanel
      title="No preview request"
      body="Open this window from QuickPanel to prepare a timesheet preview."
    />
  {/if}

  <FontScaleIndicator
    visible={controller.fontScaleIndicator.visible}
    scale={controller.fontScaleIndicator.scale}
  />
</main>
