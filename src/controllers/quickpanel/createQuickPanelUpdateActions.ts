import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

import { createLogger } from "../../lib/logger";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { QuickPanelState } from "./quickPanelTypes";

const log = createLogger("quickpanel.update");

type CreateQuickPanelUpdateActionsArgs = {
  state: QuickPanelState;
  quickPanelBridge: QuickPanelBridge;
  getPendingUpdate: () => Update | null;
  setPendingUpdate: (value: Update | null) => void;
};

export function createQuickPanelUpdateActions(
  args: CreateQuickPanelUpdateActionsArgs
) {
  const {
    state,
    quickPanelBridge,
    getPendingUpdate,
    setPendingUpdate,
  } = args;

  async function checkForUpdate() {
    log.info("checkForUpdate");

    try {
      const update = await check();
      if (update) {
        setPendingUpdate(update);
        state.updateVersion = update.version;
        state.updateStatus = "available";
        await quickPanelBridge.setUpdateAvailable(true);
        log.info("updateAvailable", { version: update.version });
        return;
      }

      setPendingUpdate(null);
      state.updateVersion = "";
      state.updateStatus = "idle";
      await quickPanelBridge.setUpdateAvailable(false);
    } catch (error) {
      log.warn("checkForUpdate failed", error);
      state.updateStatus = "idle";
      await quickPanelBridge.setUpdateAvailable(false).catch(() => {});
    }
  }

  async function openUpdatePrompt() {
    if (
      !getPendingUpdate() &&
      state.updateStatus !== "downloading" &&
      state.updateStatus !== "ready"
    ) {
      await checkForUpdate();
    }

    state.updatePromptOpen = true;
  }

  function closeUpdatePrompt() {
    if (
      state.updateStatus !== "downloading" &&
      state.updateStatus !== "ready"
    ) {
      state.updatePromptOpen = false;
    }
  }

  async function installUpdate() {
    const pendingUpdate = getPendingUpdate();
    if (!pendingUpdate) {
      return;
    }

    log.info("installUpdate", { version: pendingUpdate.version });
    state.updatePromptOpen = true;
    state.updateStatus = "downloading";
    state.updateProgress = 0;

    let totalBytes = 0;
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        totalBytes = event.data.contentLength ?? 0;
      }

      if (event.event === "Progress" && totalBytes > 0) {
        state.updateProgress = Math.min(
          state.updateProgress + (event.data.chunkLength / totalBytes) * 100,
          100
        );
      }

      if (event.event === "Finished") {
        state.updateStatus = "ready";
      }
    });

    log.info("updateInstalledRelaunching");
    await relaunch();
  }

  return {
    checkForUpdate,
    openUpdatePrompt,
    closeUpdatePrompt,
    installUpdate,
  };
}
