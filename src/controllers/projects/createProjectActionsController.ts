import { createLogger } from "../../lib/logger";
import type {
  TimesheetFormat,
  TimesheetRange,
} from "../../models/types";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { TimesheetBridge } from "../../services/bridge/timesheetBridge";
import type { QuickPanelState, QuickPanelView } from "../quickpanel/quickPanelTypes";

const log = createLogger("quickpanel.projects");

type CreateProjectActionsControllerArgs = {
  state: QuickPanelState;
  view: QuickPanelView;
  quickPanelBridge: QuickPanelBridge;
  timesheetBridge: TimesheetBridge;
  refreshFromCommand: <T>(
    command: Promise<T>,
    options?: { preserveMode?: boolean }
  ) => Promise<T>;
  queueSettingsSave: () => void;
  getManualOrder: () => string[];
  setManualOrder: (value: string[]) => void;
};

export function createProjectActionsController(
  args: CreateProjectActionsControllerArgs
) {
  const {
    state,
    view,
    quickPanelBridge,
    timesheetBridge,
    refreshFromCommand,
    queueSettingsSave,
    getManualOrder,
    setManualOrder,
  } = args;

  async function selectProject(project: string) {
    log.info("selectProject", { project });
    await refreshFromCommand(quickPanelBridge.selectProject(project), {
      preserveMode: true,
    });
  }

  async function addProject() {
    const nextValue = state.newProjectName.trim();
    if (!nextValue) {
      return;
    }

    log.info("addProject", { length: nextValue.length });
    await refreshFromCommand(quickPanelBridge.addProject(nextValue), {
      preserveMode: true,
    });
    state.newProjectName = "";
  }

  async function trackQuick(addToo = false) {
    const nextValue = state.quickName.trim();
    if (!nextValue) {
      return;
    }

    log.info("trackQuick", { addToo, length: nextValue.length });

    if (addToo) {
      await refreshFromCommand(quickPanelBridge.addProject(nextValue), {
        preserveMode: true,
      });
    }

    await refreshFromCommand(quickPanelBridge.quickProject(nextValue), {
      preserveMode: true,
    });
    state.quickName = "";
  }

  async function saveComment() {
    log.info("saveComment", {
      length: state.commentText.trim().length,
    });
    await refreshFromCommand(
      quickPanelBridge.setComment(state.commentText.trim()),
      { preserveMode: true }
    );
  }

  async function clearComment() {
    state.commentText = "";
    log.info("clearComment");
    await refreshFromCommand(quickPanelBridge.setComment(""), {
      preserveMode: true,
    });
  }

  async function removeProject(project: string) {
    log.warn("removeProject", { project });
    await refreshFromCommand(quickPanelBridge.removeProject(project), {
      preserveMode: true,
    });
  }

  async function saveAdhocProject(project: string) {
    log.info("saveAdhocProject", { project });
    await refreshFromCommand(quickPanelBridge.addProject(project), {
      preserveMode: true,
    });
  }

  async function openTimesheetPreview(
    range: TimesheetRange,
    format: TimesheetFormat = "full"
  ) {
    await timesheetBridge.openTimesheetPreviewWindow(range, format);
  }

  async function resetTimesheet() {
    if (!confirm("Reset the timesheet?")) {
      return;
    }

    log.warn("resetTimesheet confirmed");
    await refreshFromCommand(timesheetBridge.resetTimesheet(), {
      preserveMode: true,
    });
  }

  async function resetProjects() {
    if (!confirm("Reset all saved projects?")) {
      return;
    }

    log.warn("resetProjects confirmed");
    await refreshFromCommand(quickPanelBridge.resetProjects(), {
      preserveMode: true,
    });
  }

  function setCommentText(value: string) {
    state.commentText = value;
  }

  function setNewProjectName(value: string) {
    state.newProjectName = value;
  }

  function setQuickName(value: string) {
    state.quickName = value;
  }

  function handleDragStart(project: string) {
    if (view.effectiveSortMode !== "manual") {
      return;
    }

    state.draggedProject = project;
    state.dropTargetProject = project;
    state.dropPosition = "before";
  }

  function handleDragOver(event: MouseEvent, project: string) {
    if (view.effectiveSortMode !== "manual" || !state.draggedProject) {
      return;
    }

    const row = event.currentTarget as HTMLElement | null;
    if (row) {
      const rect = row.getBoundingClientRect();
      state.dropPosition =
        event.clientY < rect.top + rect.height / 2 ? "before" : "after";
    }

    state.dropTargetProject = project;
  }

  function finishDrag() {
    if (!state.draggedProject) {
      return;
    }

    const droppedProject = state.draggedProject;
    const project = state.dropTargetProject;

    if (!project || droppedProject === project) {
      state.draggedProject = null;
      state.dropTargetProject = null;
      return;
    }

    const order = [...getManualOrder()];
    const from = order.indexOf(droppedProject);
    const targetIndex = order.indexOf(project);
    let to = targetIndex;

    if (from === -1 || to === -1) {
      state.draggedProject = null;
      state.dropTargetProject = null;
      state.dropPosition = "before";
      return;
    }

    if (state.dropPosition === "after") {
      to += 1;
    }

    order.splice(from, 1);
    if (from < to) {
      to -= 1;
    }

    order.splice(to, 0, droppedProject);
    setManualOrder(order);
    queueSettingsSave();
    state.draggedProject = null;
    state.dropTargetProject = null;
    state.dropPosition = "before";
  }

  return {
    selectProject,
    addProject,
    trackQuick,
    saveComment,
    clearComment,
    removeProject,
    saveAdhocProject,
    openTimesheetPreview,
    resetTimesheet,
    resetProjects,
    setCommentText,
    setNewProjectName,
    setQuickName,
    handleDragStart,
    handleDragOver,
    finishDrag,
  };
}
