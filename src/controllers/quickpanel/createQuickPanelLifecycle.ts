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

const SCALE_STEP = 0.05;
const SCALE_MIN = 0.5;
const SCALE_MAX = 2.0;

type CreateQuickPanelLifecycleArgs = {
  state: QuickPanelState;
  currentWindow: Window;
  quickPanelBridge: QuickPanelBridge;
  stateSync: {
    loadState: (options?: { preserveMode?: boolean }) => Promise<void>;
    consumeIgnoredStateChangedEvent: () => boolean;
    dispose: () => void;
    getUiFontScale: () => number;
    setUiFontScale: (value: number) => void;
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
  onFontScaleIndicator?: (scale: number) => void;
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
    onFontScaleIndicator,
  } = args;

  function mount(options: MountOptions = {}) {
    let disposed = false;

    let ctrlHeld = false;
    function handleKeyDown(e: KeyboardEvent) { if (e.key === "Control") ctrlHeld = true; }
    function handleKeyUp(e: KeyboardEvent) { if (e.key === "Control") ctrlHeld = false; }

    function handleWheel(event: WheelEvent) {
      if (!event.ctrlKey && !ctrlHeld) return;
      event.preventDefault();
      const delta = event.deltaY < 0 ? SCALE_STEP : -SCALE_STEP;
      const next = Math.round(
        Math.min(SCALE_MAX, Math.max(SCALE_MIN, stateSync.getUiFontScale() + delta)) * 100
      ) / 100;
      stateSync.setUiFontScale(next);
      document.documentElement.style.setProperty("--font-scale", String(next));
      onFontScaleIndicator?.(next);
      void quickPanelBridge.setUiFontScale(next).catch(() => {});
    }

    window.addEventListener("keydown", handleKeyDown, { capture: true });
    window.addEventListener("keyup", handleKeyUp, { capture: true });
    window.addEventListener("wheel", handleWheel, { passive: false });

    let cleanup = () => {
      stateSync.dispose();
      window.removeEventListener("keydown", handleKeyDown, { capture: true });
      window.removeEventListener("keyup", handleKeyUp, { capture: true });
      window.removeEventListener("wheel", handleWheel);
    };

    void (async () => {
      log.info("mounted");
      await stateSync.loadState();
      document.documentElement.style.setProperty(
        "--font-scale",
        String(stateSync.getUiFontScale())
      );
      await shellActions.restoreQuickPanelBounds();
      await shellActions.applyQuickpanelModeLayout();
      await currentWindow
        .setAlwaysOnTop(state.alwaysOnTop)
        .catch(() => {});
      await currentWindow
        .setSkipTaskbar(state.alwaysOnTop)
        .catch(() => {});
      await updateActions.checkForUpdate().catch(() => {});

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
          if (!(await currentWindow.isVisible().catch(() => false))) return;
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
        window.removeEventListener("keydown", handleKeyDown, { capture: true });
        window.removeEventListener("keyup", handleKeyUp, { capture: true });
        window.removeEventListener("wheel", handleWheel);
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
