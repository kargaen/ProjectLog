import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { type Update } from "@tauri-apps/plugin-updater";

import type {
  ProjectState,
  QuickPanelMode,
  SortMode,
} from "../../models/types";
import {
  createQuickPanelBridge,
  type QuickPanelBridge,
} from "../../services/bridge/quickPanelBridge";
import {
  createTimesheetBridge,
  type TimesheetBridge,
} from "../../services/bridge/timesheetBridge";
import { createProjectActionsController } from "../projects/createProjectActionsController";
import { createSettingsActionsController } from "../settings/createSettingsActionsController";
import { createQuickPanelDialogActions } from "./createQuickPanelDialogActions";
import { createQuickPanelLifecycle } from "./createQuickPanelLifecycle";
import { createQuickPanelShellActions } from "./createQuickPanelShellActions";
import { createQuickPanelStateSync } from "./createQuickPanelStateSync";
import { createQuickPanelUpdateActions } from "./createQuickPanelUpdateActions";
import type {
  QuickPanelState,
  QuickPanelView,
  UpdateStatus,
} from "./quickPanelTypes";

const MIN_WINDOW_WIDTH = 220;
const MIN_WINDOW_HEIGHT = 90;
const AUTO_COMPACT_HEIGHT = 430;
const MIN_OPACITY = 0.35;

function createEmptyProjectState(): ProjectState {
  return {
    app_version: "",
    active_project: "",
    active_comment: "",
    projects: [],
    adhoc_projects: [],
    update_available: false,
    settings: {
      always_on_top: false,
      open_on_start: false,
      quickpanel_x: null,
      quickpanel_y: null,
      quickpanel_width: null,
      quickpanel_height: null,
      quickpanel_opacity: 1,
      project_sort_mode: "manual",
      quickpanel_mode: "normal",
      project_manual_order: [],
      project_recent_usage: {},
      timesheet_rounding_enabled: false,
      ui_font_scale: 1,
    },
  };
}

type CreateQuickPanelControllerDeps = {
  quickPanelBridge?: QuickPanelBridge;
  timesheetBridge?: TimesheetBridge;
};

export function createQuickPanelController(
  deps: CreateQuickPanelControllerDeps = {}
) {
  const quickPanelBridge =
    deps.quickPanelBridge ?? createQuickPanelBridge();
  const timesheetBridge = deps.timesheetBridge ?? createTimesheetBridge();
  const currentWindow: Window = getCurrentWindow();

  const state = $state({
    appState: createEmptyProjectState(),
    quickName: "",
    newProjectName: "",
    commentText: "",
    alwaysOnTop: false,
    openOnStart: false,
    updateStatus: "idle" as UpdateStatus,
    updateVersion: "",
    updateProgress: 0,
    updatePromptOpen: false,
    quickPanelOpacity: 1,
    sortMode: "manual" as SortMode,
    quickPanelMode: "normal" as QuickPanelMode,
    draggedProject: null as string | null,
    dropTargetProject: null as string | null,
    dropPosition: "before" as "before" | "after",
    dialogOpen: false,
    dialogMode: "",
    aboutOpen: false,
    dialogTitle: "",
    dialogValue: "",
    closeInputOnSubmit: true,
  }) as QuickPanelState;

  let pendingUpdate = $state<Update | null>(null);
  let recentProjects = $state<Record<string, number>>({});
  let manualOrder = $state<string[]>([]);
  let timesheetRoundingEnabled = $state(false);
  let uiFontScale = $state(1);
  let fontScaleIndicator = $state({ visible: false, scale: 1 });
  let fontScaleIndicatorTimer: ReturnType<typeof setTimeout> | undefined;
  let currentWindowHeight = $state(Infinity);
  const stateSync = createQuickPanelStateSync({
    state,
    quickPanelBridge,
    getManualOrder: () => manualOrder,
    setManualOrder: (value) => {
      manualOrder = value;
    },
    getRecentProjects: () => recentProjects,
    setRecentProjects: (value) => {
      recentProjects = value;
    },
    getTimesheetRoundingEnabled: () => timesheetRoundingEnabled,
    setTimesheetRoundingEnabled: (value) => {
      timesheetRoundingEnabled = value;
    },
    getUiFontScale: () => uiFontScale,
    setUiFontScale: (value) => {
      uiFontScale = value;
    },
  });

  const view = {
    get minOpacity() {
      return MIN_OPACITY;
    },
    get isCompactLayout() {
      return (
        state.quickPanelMode === "compact" ||
        currentWindowHeight < AUTO_COMPACT_HEIGHT
      );
    },
    get effectiveSortMode(): SortMode {
      return this.isCompactLayout ? "manual" : state.sortMode;
    },
    get allProjects() {
      const combined = [
        ...state.appState.projects,
        ...state.appState.adhoc_projects,
      ];

      if (this.effectiveSortMode === "alphabetical") {
        return [...combined].sort((a, b) => a.localeCompare(b));
      }

      if (this.effectiveSortMode === "recent") {
        const recentProjects = stateSync.getRecentProjects();
        return [...combined].sort(
          (a, b) =>
            (recentProjects[b] ?? 0) - (recentProjects[a] ?? 0) ||
            a.localeCompare(b)
        );
      }

      return stateSync
        .getManualOrder()
        .filter((project) => combined.includes(project));
    },
  } as QuickPanelView;

  const shellActions = createQuickPanelShellActions({
    state,
    currentWindow,
    quickPanelBridge,
    minWindowWidth: MIN_WINDOW_WIDTH,
    minWindowHeight: MIN_WINDOW_HEIGHT,
    setCurrentWindowHeight: (value) => {
      currentWindowHeight = value;
    },
  });

  const settingsActions = createSettingsActionsController({
    state,
    currentWindow,
    persistUiSettings: stateSync.persistUiSettings,
    applyQuickpanelModeLayout: shellActions.applyQuickpanelModeLayout,
    queueSettingsSave: stateSync.queueSettingsSave,
  });

  const projectActions = createProjectActionsController({
    state,
    view,
    quickPanelBridge,
    timesheetBridge,
    refreshFromCommand: stateSync.refreshFromCommand,
    queueSettingsSave: stateSync.queueSettingsSave,
    getManualOrder: stateSync.getManualOrder,
    setManualOrder: stateSync.setManualOrder,
  });

  const dialogActions = createQuickPanelDialogActions({
    state,
    quickPanelBridge,
    loadState: stateSync.loadState,
  });

  const updateActions = createQuickPanelUpdateActions({
    state,
    quickPanelBridge,
    getPendingUpdate: () => pendingUpdate,
    setPendingUpdate: (value) => {
      pendingUpdate = value;
    },
  });

  const lifecycle = createQuickPanelLifecycle({
    state,
    currentWindow,
    quickPanelBridge,
    stateSync: {
      ...stateSync,
      getUiFontScale: stateSync.getUiFontScale,
      setUiFontScale: (value) => {
        uiFontScale = value;
      },
    },
    shellActions,
    dialogActions,
    updateActions,
    onFontScaleIndicator: (scale) => {
      fontScaleIndicator = { visible: true, scale };
      clearTimeout(fontScaleIndicatorTimer);
      fontScaleIndicatorTimer = setTimeout(() => {
        fontScaleIndicator = { ...fontScaleIndicator, visible: false };
      }, 1200);
    },
  });

  return {
    state,
    view,
    get uiFontScale() { return uiFontScale; },
    get fontScaleIndicator() { return fontScaleIndicator; },
    mount: lifecycle.mount,
    startWindowDrag: shellActions.startWindowDrag,
    hideWindow: shellActions.hideWindow,
    selectProject: projectActions.selectProject,
    addProject: projectActions.addProject,
    trackQuick: projectActions.trackQuick,
    saveComment: projectActions.saveComment,
    clearComment: projectActions.clearComment,
    removeProject: projectActions.removeProject,
    saveAdhocProject: projectActions.saveAdhocProject,
    openTimesheetPreview: projectActions.openTimesheetPreview,
    openLogFile: shellActions.openLogFile,
    resetTimesheet: projectActions.resetTimesheet,
    resetProjects: projectActions.resetProjects,
    toggleAlwaysOnTop: settingsActions.toggleAlwaysOnTop,
    toggleOpenOnStart: settingsActions.toggleOpenOnStart,
    toggleQuickPanelMode: settingsActions.toggleQuickPanelMode,
    openUpdatePrompt: updateActions.openUpdatePrompt,
    closeUpdatePrompt: updateActions.closeUpdatePrompt,
    installUpdate: updateActions.installUpdate,
    submitDialog: dialogActions.submitDialog,
    cancelDialog: dialogActions.cancelDialog,
    openAbout: dialogActions.openAbout,
    closeAbout: dialogActions.closeAbout,
    setCommentText: projectActions.setCommentText,
    setNewProjectName: projectActions.setNewProjectName,
    setQuickName: projectActions.setQuickName,
    setDialogValue: dialogActions.setDialogValue,
    handleGlobalKeydown: dialogActions.handleGlobalKeydown,
    setSortMode: settingsActions.setSortMode,
    setQuickPanelOpacity: settingsActions.setQuickPanelOpacity,
    handleDragStart: projectActions.handleDragStart,
    handleDragOver: projectActions.handleDragOver,
    finishDrag: projectActions.finishDrag,
    openDiagnosticLog: dialogActions.openDiagnosticLog,
    openFeedback: dialogActions.openFeedback,
    openProjectHomepage: dialogActions.openProjectHomepage,
    openPortfolio: dialogActions.openPortfolio,
    openReleaseNotes: dialogActions.openReleaseNotes,
  };
}
