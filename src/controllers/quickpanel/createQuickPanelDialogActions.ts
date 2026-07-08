import { NEW_GROUP_DIALOG_PREFIX } from "../projects/createProjectContextMenuController";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { QuickPanelState } from "./quickPanelTypes";

type CreateQuickPanelDialogActionsArgs = {
  state: QuickPanelState;
  quickPanelBridge: QuickPanelBridge;
  loadState: (options?: { preserveMode?: boolean }) => Promise<void>;
};

export function createQuickPanelDialogActions(
  args: CreateQuickPanelDialogActionsArgs
) {
  const { state, quickPanelBridge, loadState } = args;

  async function submitDialog() {
    const mode = state.dialogMode;
    const value = state.dialogValue.trim();

    if (mode.startsWith(NEW_GROUP_DIALOG_PREFIX)) {
      const project = mode.slice(NEW_GROUP_DIALOG_PREFIX.length);
      if (value && project) {
        await quickPanelBridge.setProjectGroup(project, value);
      }
    } else {
      await quickPanelBridge.submitInput(mode, value);
    }

    state.dialogOpen = false;
    state.dialogMode = "";
    state.dialogValue = "";
    await loadState();
  }

  function cancelDialog() {
    state.dialogOpen = false;
    state.dialogMode = "";
    state.dialogValue = "";
    state.closeInputOnSubmit = true;
  }

  function openAbout() {
    state.aboutOpen = true;
  }

  function closeAbout() {
    state.aboutOpen = false;
  }

  function setDialogValue(value: string) {
    state.dialogValue = value;
  }

  function handleGlobalKeydown(event: KeyboardEvent) {
    if (state.dialogOpen && event.key === "Enter") {
      void submitDialog();
    }

    if (state.dialogOpen && event.key === "Escape") {
      cancelDialog();
    }

    if (state.contextMenuProject && event.key === "Escape") {
      state.contextMenuProject = null;
      state.contextMenuPosition = null;
    }
  }

  async function openDiagnosticLog() {
    await quickPanelBridge.openDiagnosticLog();
  }

  async function openGithubIssues() {
    await quickPanelBridge.openGithubIssues();
  }

  async function openProjectHomepage() {
    await quickPanelBridge.openProjectHomepage();
  }

  async function openPortfolio() {
    await quickPanelBridge.openPortfolio();
  }

  async function openReleaseNotes() {
    await quickPanelBridge.openReleaseNotes();
  }

  return {
    submitDialog,
    cancelDialog,
    openAbout,
    closeAbout,
    setDialogValue,
    handleGlobalKeydown,
    openDiagnosticLog,
    openGithubIssues,
    openProjectHomepage,
    openPortfolio,
    openReleaseNotes,
  };
}
