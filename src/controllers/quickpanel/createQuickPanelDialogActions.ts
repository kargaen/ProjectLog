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
    await quickPanelBridge.submitInput(
      state.dialogMode,
      state.dialogValue.trim()
    );
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
  }

  async function openDiagnosticLog() {
    await quickPanelBridge.openDiagnosticLog();
  }

  async function openFeedback() {
    await quickPanelBridge.openFeedback();
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
    openFeedback,
    openProjectHomepage,
    openPortfolio,
    openReleaseNotes,
  };
}
