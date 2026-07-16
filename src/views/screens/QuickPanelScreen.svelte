<script lang="ts">
  import type { SortMode } from "../../models/types";
  import type { ProjectListEntry } from "../../controllers/quickpanel/quickPanelTypes";
  import AboutDialog from "../components/dialogs/AboutDialog.view.svelte";
  import InputDialog from "../components/dialogs/InputDialog.view.svelte";
  import UpdateDialog from "../components/dialogs/UpdateDialog.view.svelte";
  import CompactModeFooter from "../components/quickpanel/CompactModeFooter.view.svelte";
  import QuickPanelControls from "../components/quickpanel/QuickPanelControls.view.svelte";
  import QuickPanelHeader from "../components/quickpanel/QuickPanelHeader.view.svelte";
  import ProjectListPanel from "../components/projects/ProjectListPanel.view.svelte";
  import FontScaleIndicator from "../components/shared/FontScaleIndicator.view.svelte";

  type UpdateStatus = "idle" | "available" | "downloading" | "ready";

  let {
    appIcon,
    activeProject,
    activeComment,
    appVersion,
    permanentProjects,
    allProjects,
    projectListEntries,
    groupProjectsEnabled,
    hasProjectGroups,
    knownGroupNames,
    projectColors,
    projectGroups,
    contextMenuProject,
    contextMenuPosition,
    sortMode,
    effectiveSortMode,
    isCompactLayout,
    draggedProject,
    dropTargetProject,
    dropPosition,
    commentText,
    newProjectName,
    quickName,
    alwaysOnTop,
    openOnStart,
    minOpacity,
    quickPanelOpacity,
    updateStatus,
    updateVersion,
    updateProgress,
    dialogOpen,
    dialogTitle,
    dialogValue,
    aboutOpen,
    updatePromptOpen,
    onStartDrag,
    onHide,
    onSetSortMode,
    onToggleGroupProjectsEnabled,
    onToggleProjectGroupCollapsed,
    onHandleDragOver,
    onHandleDragStart,
    onSelectProject,
    onRemoveProject,
    onSaveAdhocProject,
    onOpenContextMenu,
    onCloseContextMenu,
    onPickColor,
    onPickGroup,
    onRequestNewGroup,
    onCommentTextChange,
    onSaveComment,
    onClearComment,
    onNewProjectNameChange,
    onAddProject,
    onQuickNameChange,
    onTrackQuick,
    onOpenTimesheetPreview,
    onOpenLogFile,
    onResetTimesheet,
    onResetProjects,
    onToggleAlwaysOnTop,
    onToggleOpenOnStart,
    onQuickPanelOpacityChange,
    onOpenAbout,
    onToggleQuickPanelMode,
    onDialogValueChange,
    onCancelDialog,
    onSubmitDialog,
    onOpenProjectHomepage,
    onOpenGithubIssues,
    onOpenPortfolio,
    onOpenDiagnosticLog,
    onCloseAbout,
    onOpenUpdatePrompt,
    onOpenReleaseNotes,
    onCloseUpdatePrompt,
    onInstallUpdate,
    fontScaleIndicator,
  }: {
    appIcon: string;
    activeProject: string;
    activeComment: string;
    appVersion: string;
    permanentProjects: string[];
    allProjects: string[];
    projectListEntries: ProjectListEntry[];
    groupProjectsEnabled: boolean;
    hasProjectGroups: boolean;
    knownGroupNames: string[];
    projectColors: Record<string, string>;
    projectGroups: Record<string, string>;
    contextMenuProject: string | null;
    contextMenuPosition: { x: number; y: number } | null;
    sortMode: SortMode;
    effectiveSortMode: SortMode;
    isCompactLayout: boolean;
    draggedProject: string | null;
    dropTargetProject: string | null;
    dropPosition: "before" | "after";
    commentText: string;
    newProjectName: string;
    quickName: string;
    alwaysOnTop: boolean;
    openOnStart: boolean;
    minOpacity: number;
    quickPanelOpacity: number;
    updateStatus: UpdateStatus;
    updateVersion: string;
    updateProgress: number;
    dialogOpen: boolean;
    dialogTitle: string;
    dialogValue: string;
    aboutOpen: boolean;
    updatePromptOpen: boolean;
    onStartDrag: () => void | Promise<void>;
    onHide: () => void | Promise<void>;
    onSetSortMode: (mode: SortMode) => void;
    onToggleGroupProjectsEnabled: () => void;
    onToggleProjectGroupCollapsed: (groupName: string) => void;
    onHandleDragOver: (event: MouseEvent, project: string) => void;
    onHandleDragStart: (project: string) => void;
    onSelectProject: (project: string) => void | Promise<void>;
    onRemoveProject: (project: string) => void | Promise<void>;
    onSaveAdhocProject: (project: string) => void | Promise<void>;
    onOpenContextMenu: (project: string, x: number, y: number) => void;
    onCloseContextMenu: () => void;
    onPickColor: (color: string | null) => void | Promise<void>;
    onPickGroup: (group: string | null) => void | Promise<void>;
    onRequestNewGroup: () => void;
    onCommentTextChange: (value: string) => void;
    onSaveComment: () => void | Promise<void>;
    onClearComment: () => void | Promise<void>;
    onNewProjectNameChange: (value: string) => void;
    onAddProject: () => void | Promise<void>;
    onQuickNameChange: (value: string) => void;
    onTrackQuick: () => void | Promise<void>;
    onOpenTimesheetPreview: (
      range: "today" | "week" | "all",
      format?: "full" | "recent"
    ) => void | Promise<void>;
    onOpenLogFile: () => void | Promise<void>;
    onResetTimesheet: () => void | Promise<void>;
    onResetProjects: () => void | Promise<void>;
    onToggleAlwaysOnTop: () => void | Promise<void>;
    onToggleOpenOnStart: () => void | Promise<void>;
    onQuickPanelOpacityChange: (value: number) => void;
    onOpenAbout: () => void;
    onToggleQuickPanelMode: () => void | Promise<void>;
    onDialogValueChange: (value: string) => void;
    onCancelDialog: () => void | Promise<void>;
    onSubmitDialog: () => void | Promise<void>;
    onOpenProjectHomepage: () => void | Promise<void>;
    onOpenGithubIssues: () => void | Promise<void>;
    onOpenPortfolio: () => void | Promise<void>;
    onOpenDiagnosticLog: () => void | Promise<void>;
    onCloseAbout: () => void | Promise<void>;
    onOpenUpdatePrompt: () => void | Promise<void>;
    onOpenReleaseNotes: () => void | Promise<void>;
    onCloseUpdatePrompt: () => void | Promise<void>;
    onInstallUpdate: () => void | Promise<void>;
    fontScaleIndicator: { visible: boolean; scale: number };
  } = $props();
</script>

<main class:compact-shell={isCompactLayout} class="quickpanel-screen" style:opacity={quickPanelOpacity}>
  {#if updateStatus !== "idle"}
    <section class="update">
      <div>
        {#if updateStatus === "available"}Update available:
          v{updateVersion}{/if}
        {#if updateStatus === "downloading"}Updating...
          {Math.round(updateProgress)}%{/if}
        {#if updateStatus === "ready"}Update installed. Restarting...{/if}
      </div>
      {#if updateStatus === "available"}
        <button class="primary small" onclick={onOpenUpdatePrompt}>
          Details
        </button>
      {/if}
    </section>
  {/if}

  <QuickPanelHeader
    {appIcon}
    {activeProject}
    activeComment={activeComment}
    showActiveComment={!isCompactLayout}
    onStartDrag={onStartDrag}
    onHide={onHide}
  />

  <ProjectListPanel
    {isCompactLayout}
    {sortMode}
    {allProjects}
    {projectListEntries}
    {groupProjectsEnabled}
    {hasProjectGroups}
    {knownGroupNames}
    {projectColors}
    {projectGroups}
    {activeProject}
    {permanentProjects}
    {effectiveSortMode}
    {draggedProject}
    {dropTargetProject}
    {dropPosition}
    {contextMenuProject}
    {contextMenuPosition}
    onSetSortMode={onSetSortMode}
    onToggleGroupProjectsEnabled={onToggleGroupProjectsEnabled}
    onToggleProjectGroupCollapsed={onToggleProjectGroupCollapsed}
    onHandleDragOver={onHandleDragOver}
    onHandleDragStart={onHandleDragStart}
    onSelectProject={onSelectProject}
    onRemoveProject={onRemoveProject}
    onSaveAdhocProject={onSaveAdhocProject}
    onOpenContextMenu={onOpenContextMenu}
    onCloseContextMenu={onCloseContextMenu}
    onPickColor={onPickColor}
    onPickGroup={onPickGroup}
    onRequestNewGroup={onRequestNewGroup}
  />

  {#if !isCompactLayout}
    <QuickPanelControls
      {activeProject}
      {commentText}
      {newProjectName}
      {quickName}
      {alwaysOnTop}
      {openOnStart}
      {minOpacity}
      {quickPanelOpacity}
      onCommentTextChange={onCommentTextChange}
      onSaveComment={onSaveComment}
      onClearComment={onClearComment}
      onNewProjectNameChange={onNewProjectNameChange}
      onAddProject={onAddProject}
      onQuickNameChange={onQuickNameChange}
      onTrackQuick={onTrackQuick}
      onOpenTimesheetPreview={onOpenTimesheetPreview}
      onOpenLogFile={onOpenLogFile}
      onResetTimesheet={onResetTimesheet}
      onResetProjects={onResetProjects}
      onToggleAlwaysOnTop={onToggleAlwaysOnTop}
      onToggleOpenOnStart={onToggleOpenOnStart}
      onQuickPanelOpacityChange={onQuickPanelOpacityChange}
      onOpenAbout={onOpenAbout}
    />
  {/if}

  <CompactModeFooter
    {isCompactLayout}
    onToggleQuickPanelMode={onToggleQuickPanelMode}
  />

  {#if dialogOpen}
    <InputDialog
      title={dialogTitle}
      value={dialogValue}
      onValueChange={onDialogValueChange}
      onCancel={onCancelDialog}
      onSubmit={onSubmitDialog}
    />
  {/if}

  {#if aboutOpen}
    <AboutDialog
      version={appVersion}
      onOpenProjectHomepage={onOpenProjectHomepage}
      onOpenGithubIssues={onOpenGithubIssues}
      onOpenPortfolio={onOpenPortfolio}
      onOpenDiagnosticLog={onOpenDiagnosticLog}
      onClose={onCloseAbout}
    />
  {/if}

  {#if updatePromptOpen}
    <UpdateDialog
      {updateStatus}
      {updateVersion}
      {updateProgress}
      onOpenReleaseNotes={onOpenReleaseNotes}
      onClose={onCloseUpdatePrompt}
      onInstallUpdate={onInstallUpdate}
    />
  {/if}

  <FontScaleIndicator
    visible={fontScaleIndicator.visible}
    scale={fontScaleIndicator.scale}
  />
</main>
