import { createLogger } from "../../lib/logger";
import type { ProjectState } from "../../models/types";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { QuickPanelState } from "./quickPanelTypes";

const log = createLogger("quickpanel.state");

type CreateQuickPanelStateSyncArgs = {
  state: QuickPanelState;
  quickPanelBridge: QuickPanelBridge;
  getManualOrder: () => string[];
  setManualOrder: (value: string[]) => void;
  getRecentProjects: () => Record<string, number>;
  setRecentProjects: (value: Record<string, number>) => void;
  getTimesheetRoundingEnabled: () => boolean;
  setTimesheetRoundingEnabled: (value: boolean) => void;
};

export function createQuickPanelStateSync(
  args: CreateQuickPanelStateSyncArgs
) {
  const {
    state,
    quickPanelBridge,
    getManualOrder,
    setManualOrder,
    getRecentProjects,
    setRecentProjects,
    getTimesheetRoundingEnabled,
    setTimesheetRoundingEnabled,
  } = args;

  let ignoredStateChangedEvents = 0;
  let settingsSaveTimer: ReturnType<typeof setTimeout> | undefined;

  function syncManualOrder(projects: string[]) {
    const currentOrder = getManualOrder();
    const seen = new Set(projects);
    const ordered = currentOrder.filter((project) => seen.has(project));

    for (const project of projects) {
      if (!ordered.includes(project)) {
        ordered.push(project);
      }
    }

    if (
      ordered.length !== currentOrder.length ||
      ordered.some((project, index) => project !== currentOrder[index])
    ) {
      setManualOrder(ordered);
      queueSettingsSave();
    }
  }

  function applyAppState(
    nextState: ProjectState,
    options?: { preserveMode?: boolean }
  ) {
    const preserveMode = options?.preserveMode ?? false;
    const currentQuickPanelMode = state.quickPanelMode;
    const currentSortMode = state.sortMode;

    state.appState = nextState;
    state.commentText = nextState.active_comment;
    state.alwaysOnTop = nextState.settings.always_on_top;
    state.openOnStart = nextState.settings.open_on_start;
    state.quickPanelOpacity = nextState.settings.quickpanel_opacity;
    state.sortMode = preserveMode
      ? currentSortMode
      : nextState.settings.project_sort_mode ?? "manual";
    state.quickPanelMode = preserveMode
      ? currentQuickPanelMode
      : nextState.settings.quickpanel_mode ?? "normal";
    setManualOrder(nextState.settings.project_manual_order ?? []);
    setRecentProjects(nextState.settings.project_recent_usage ?? {});
    setTimesheetRoundingEnabled(
      nextState.settings.timesheet_rounding_enabled ?? false
    );

    syncManualOrder([
      ...nextState.projects,
      ...nextState.adhoc_projects,
    ]);
  }

  async function loadState(options?: { preserveMode?: boolean }) {
    log.debug("loadState");
    const nextState = await quickPanelBridge.getState();
    applyAppState(nextState, options);
  }

  async function refreshFromCommand<T>(
    command: Promise<T>,
    options?: { preserveMode?: boolean }
  ) {
    ignoredStateChangedEvents += 1;
    const result = await command;
    await loadState(options);
    return result;
  }

  async function persistUiSettings() {
    await quickPanelBridge.saveUiSettings({
      alwaysOnTop: state.alwaysOnTop,
      openOnStart: state.openOnStart,
      quickpanelOpacity: state.quickPanelOpacity,
      projectSortMode: state.sortMode,
      quickpanelMode: state.quickPanelMode,
      projectManualOrder: getManualOrder(),
      projectRecentUsage: getRecentProjects(),
      timesheetRoundingEnabled: getTimesheetRoundingEnabled(),
    });
  }

  function queueSettingsSave() {
    if (settingsSaveTimer) {
      clearTimeout(settingsSaveTimer);
    }

    settingsSaveTimer = setTimeout(() => {
      void persistUiSettings().catch(() => {});
    }, 160);
  }

  function consumeIgnoredStateChangedEvent() {
    if (ignoredStateChangedEvents === 0) {
      return false;
    }

    ignoredStateChangedEvents -= 1;
    return true;
  }

  function dispose() {
    if (settingsSaveTimer) {
      clearTimeout(settingsSaveTimer);
    }
  }

  return {
    loadState,
    refreshFromCommand,
    persistUiSettings,
    queueSettingsSave,
    consumeIgnoredStateChangedEvent,
    getManualOrder,
    setManualOrder,
    getRecentProjects,
    dispose,
  };
}
