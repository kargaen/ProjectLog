import { listen } from "@tauri-apps/api/event";
import type { Window } from "@tauri-apps/api/window";

import { createLogger } from "../../lib/logger";
import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type {
  MountOptions,
  QuickPanelState,
  ShowInputPayload,
} from "./quickPanelTypes";

const log = createLogger("quickpanel.lifecycle");

type CreateQuickPanelLifecycleArgs = {
  state: QuickPanelState;
  currentWindow: Window;
  quickPanelBridge: QuickPanelBridge;
  stateSync: {
    loadState: (options?: { preserveMode?: boolean }) => Promise<void>;
    consumeIgnoredStateChangedEvent: () => boolean;
    dispose: () => void;
  };
  shellActions: {
    restoreQuickPanelBounds: () => Promise<void>;
    applyQuickpanelModeLayout: () => Promise<void>;
    updateWindowHeight: (nextHeight?: number) => Promise<void>;
  };
  dialogActions: {
    openAbout: () => void;
  };
  updateActions: {
    checkForUpdate: () => Promise<void>;
    openUpdatePrompt: () => Promise<void>;
  };
};

export function createQuickPanelLifecycle(
  args: CreateQuickPanelLifecycleArgs
) {
  const {
    state,
    currentWindow,
    quickPanelBridge,
    stateSync,
    shellActions,
    dialogActions,
    updateActions,
  } = args;

  function mount(options: MountOptions = {}) {
    let disposed = false;
    let cleanup = () => stateSync.dispose();

    void (async () => {
      log.info("mounted");
      await stateSync.loadState();
      await shellActions.restoreQuickPanelBounds();
      await shellActions.applyQuickpanelModeLayout();
      await currentWindow
        .setAlwaysOnTop(state.alwaysOnTop)
        .catch(() => {});
      await currentWindow
        .setSkipTaskbar(state.alwaysOnTop)
        .catch(() => {});
      await updateActions.checkForUpdate();

      if (state.openOnStart) {
        await currentWindow.show().catch(() => {});
      }

      let lastBounds = "";
      const unlistenResized = currentWindow.onResized(
        async ({ payload: size }) => {
          await shellActions.updateWindowHeight(size.height).catch(() => {});
        }
      );

      const interval = setInterval(async () => {
        try {
          const pos = await currentWindow.outerPosition();
          const size = await currentWindow.outerSize();
          const next = `${pos.x}:${pos.y}:${size.width}:${size.height}`;

          if (next !== lastBounds) {
            lastBounds = next;
            await quickPanelBridge.saveQuickpanelBounds({
              x: pos.x,
              y: pos.y,
              width: size.width,
              height: size.height,
            });
          }
        } catch {
          // Ignore transient window measurement failures.
        }
      }, 800);

      const unlistenInput = listen<ShowInputPayload>(
        "show-input",
        (event) => {
          state.dialogMode = event.payload.mode;
          state.dialogTitle = event.payload.title;
          state.dialogValue = event.payload.value;
          state.closeInputOnSubmit = event.payload.closeOnSubmit ?? true;
          state.dialogOpen = true;
          setTimeout(() => options.focusDialogInput?.(), 50);
        }
      );

      const unlistenAbout = listen("show-about", () => {
        dialogActions.openAbout();
      });

      const unlistenState = listen("state-changed", () => {
        if (stateSync.consumeIgnoredStateChangedEvent()) {
          return;
        }

        void stateSync.loadState();
      });

      const unlistenUpdatePrompt = listen(
        "show-update-prompt",
        () => {
          void updateActions.openUpdatePrompt();
        }
      );

      const unlistenSubmitted = listen("input-submitted", () => {
        state.dialogOpen = false;
        state.dialogMode = "";
        state.dialogValue = "";
        if (state.closeInputOnSubmit) {
          void currentWindow.hide().catch(() => {});
        }
        state.closeInputOnSubmit = true;
      });

      cleanup = () => {
        stateSync.dispose();
        clearInterval(interval);
        void unlistenResized.then((fn) => fn());
        void unlistenInput.then((fn) => fn());
        void unlistenAbout.then((fn) => fn());
        void unlistenState.then((fn) => fn());
        void unlistenUpdatePrompt.then((fn) => fn());
        void unlistenSubmitted.then((fn) => fn());
      };

      if (disposed) {
        cleanup();
      }
    })();

    return () => {
      disposed = true;
      cleanup();
    };
  }

  return {
    mount,
  };
}
