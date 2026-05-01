<script lang="ts">
  import { onMount } from "svelte";

  import {
    createQuickPanelController,
  } from "./controllers/quickpanel/createQuickPanelController.svelte";
  import QuickPanelScreen from "./views/screens/QuickPanelScreen.svelte";
  import TimesheetScreen from "./views/screens/TimesheetScreen.svelte";
  import "./views/screens/quickpanel.css";
  import appIcon from "../icon.svg";

  const controller = createQuickPanelController();
  const state = controller.state;
  const view = controller.view;

  const windowParams = new URLSearchParams(window.location.search);
  const tauriWindowLabel =
    (window as Window & {
      __TAURI_INTERNALS__?: {
        metadata?: {
          currentWindow?: { label?: string };
        };
      };
    }).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
  const currentWindowLabel =
    tauriWindowLabel ?? windowParams.get("window") ?? "main";
  const isDedicatedTimesheetPreviewWindow =
    windowParams.get("window") === "timesheet-preview" ||
    currentWindowLabel === "timesheet-preview";

  onMount(() => {
    if (isDedicatedTimesheetPreviewWindow) {
      return;
    }

    return controller.mount({
      focusDialogInput: () => {
        document
          .querySelector<HTMLInputElement>(
            ".quickpanel-screen .dialog-input"
          )
          ?.focus();
      },
    });
  });
</script>

<svelte:window
  onkeydown={controller.handleGlobalKeydown}
  onmouseup={controller.finishDrag}
/>

{#if isDedicatedTimesheetPreviewWindow}
  <TimesheetScreen />
{:else}
  <QuickPanelScreen
    {appIcon}
    activeProject={state.appState.active_project}
    activeComment={state.appState.active_comment}
    appVersion={state.appState.app_version}
    permanentProjects={state.appState.projects}
    allProjects={view.allProjects}
    sortMode={state.sortMode}
    effectiveSortMode={view.effectiveSortMode}
    isCompactLayout={view.isCompactLayout}
    draggedProject={state.draggedProject}
    dropTargetProject={state.dropTargetProject}
    dropPosition={state.dropPosition}
    commentText={state.commentText}
    newProjectName={state.newProjectName}
    quickName={state.quickName}
    alwaysOnTop={state.alwaysOnTop}
    openOnStart={state.openOnStart}
    minOpacity={view.minOpacity}
    quickPanelOpacity={state.quickPanelOpacity}
    updateStatus={state.updateStatus}
    updateVersion={state.updateVersion}
    updateProgress={state.updateProgress}
    dialogOpen={state.dialogOpen}
    dialogTitle={state.dialogTitle}
    dialogValue={state.dialogValue}
    aboutOpen={state.aboutOpen}
    updatePromptOpen={state.updatePromptOpen}
    onStartDrag={controller.startWindowDrag}
    onHide={controller.hideWindow}
    onSetSortMode={controller.setSortMode}
    onHandleDragOver={controller.handleDragOver}
    onHandleDragStart={controller.handleDragStart}
    onSelectProject={controller.selectProject}
    onRemoveProject={controller.removeProject}
    onSaveAdhocProject={controller.saveAdhocProject}
    onCommentTextChange={controller.setCommentText}
    onSaveComment={controller.saveComment}
    onClearComment={controller.clearComment}
    onNewProjectNameChange={controller.setNewProjectName}
    onAddProject={controller.addProject}
    onQuickNameChange={controller.setQuickName}
    onTrackQuick={controller.trackQuick}
    onOpenTimesheetPreview={controller.openTimesheetPreview}
    onOpenLogFile={controller.openLogFile}
    onResetTimesheet={controller.resetTimesheet}
    onResetProjects={controller.resetProjects}
    onToggleAlwaysOnTop={controller.toggleAlwaysOnTop}
    onToggleOpenOnStart={controller.toggleOpenOnStart}
    onQuickPanelOpacityChange={controller.setQuickPanelOpacity}
    onOpenAbout={controller.openAbout}
    onToggleQuickPanelMode={controller.toggleQuickPanelMode}
    onDialogValueChange={controller.setDialogValue}
    onCancelDialog={controller.cancelDialog}
    onSubmitDialog={controller.submitDialog}
    onOpenProjectHomepage={controller.openProjectHomepage}
    onOpenFeedback={controller.openFeedback}
    onOpenPortfolio={controller.openPortfolio}
    onOpenDiagnosticLog={controller.openDiagnosticLog}
    onCloseAbout={controller.closeAbout}
    onOpenUpdatePrompt={controller.openUpdatePrompt}
    onOpenReleaseNotes={controller.openReleaseNotes}
    onCloseUpdatePrompt={controller.closeUpdatePrompt}
    onInstallUpdate={controller.installUpdate}
  />
{/if}
