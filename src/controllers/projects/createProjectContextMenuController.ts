import { createLogger } from "../../lib/logger";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { QuickPanelState } from "../quickpanel/quickPanelTypes";

const log = createLogger("quickpanel.projectContextMenu");

export const NEW_GROUP_DIALOG_PREFIX = "new_group::";

type CreateProjectContextMenuControllerArgs = {
  state: QuickPanelState;
  quickPanelBridge: QuickPanelBridge;
  refreshFromCommand: <T>(
    command: Promise<T>,
    options?: { preserveMode?: boolean }
  ) => Promise<T>;
  forceGroupProjectsEnabled: () => void;
};

export function createProjectContextMenuController(
  args: CreateProjectContextMenuControllerArgs
) {
  const { state, quickPanelBridge, refreshFromCommand, forceGroupProjectsEnabled } = args;

  function openContextMenu(project: string, x: number, y: number) {
    state.contextMenuProject = project;
    state.contextMenuPosition = { x, y };
  }

  function closeContextMenu() {
    state.contextMenuProject = null;
    state.contextMenuPosition = null;
  }

  async function pickColor(color: string | null) {
    const project = state.contextMenuProject;
    if (!project) {
      return;
    }

    log.info("pickColor", { project, color });
    closeContextMenu();
    await refreshFromCommand(
      quickPanelBridge.setProjectColor(project, color),
      { preserveMode: true }
    );
  }

  async function pickGroup(group: string | null) {
    const project = state.contextMenuProject;
    if (!project) {
      return;
    }

    log.info("pickGroup", { project, group });
    if (group) {
      forceGroupProjectsEnabled();
    }
    closeContextMenu();
    await refreshFromCommand(
      quickPanelBridge.setProjectGroup(project, group),
      { preserveMode: true }
    );
  }

  function requestNewGroup() {
    const project = state.contextMenuProject;
    if (!project) {
      return;
    }

    forceGroupProjectsEnabled();
    closeContextMenu();
    state.dialogMode = `${NEW_GROUP_DIALOG_PREFIX}${project}`;
    state.dialogTitle = "New group name:";
    state.dialogValue = "";
    state.closeInputOnSubmit = true;
    state.dialogOpen = true;
  }

  return {
    openContextMenu,
    closeContextMenu,
    pickColor,
    pickGroup,
    requestNewGroup,
  };
}
