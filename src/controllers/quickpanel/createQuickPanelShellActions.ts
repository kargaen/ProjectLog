import {
  availableMonitors,
  PhysicalPosition,
  PhysicalSize,
  primaryMonitor,
  type Window,
} from "@tauri-apps/api/window";

import type { QuickPanelBridge } from "../../services/bridge/quickPanelBridge";
import type { QuickPanelState } from "./quickPanelTypes";

type CreateQuickPanelShellActionsArgs = {
  state: QuickPanelState;
  currentWindow: Window;
  quickPanelBridge: QuickPanelBridge;
  minWindowWidth: number;
  minWindowHeight: number;
  getCurrentWindowHeight: () => number;
  setCurrentWindowHeight: (value: number) => void;
};

export function createQuickPanelShellActions(
  args: CreateQuickPanelShellActionsArgs
) {
  const {
    state,
    currentWindow,
    quickPanelBridge,
    minWindowWidth,
    minWindowHeight,
    setCurrentWindowHeight,
  } = args;

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  async function getMinimumWindowSizePhysical() {
    const scaleFactor = await currentWindow.scaleFactor().catch(() => 1);

    return {
      minWidth: Math.ceil(minWindowWidth * scaleFactor),
      minHeight: Math.ceil(minWindowHeight * scaleFactor),
    };
  }

  async function updateWindowHeight(nextHeight?: number) {
    const scaleFactor = await currentWindow.scaleFactor().catch(() => 1);
    if (nextHeight !== undefined) {
      setCurrentWindowHeight(nextHeight / scaleFactor);
      return;
    }

    const size = await currentWindow.outerSize().catch(() => null);
    if (!size) return;
    setCurrentWindowHeight(size.height / scaleFactor);
  }

  async function applyQuickpanelModeLayout() {
    await currentWindow
      .setSizeConstraints({
        minWidth: minWindowWidth,
        minHeight: minWindowHeight,
      })
      .catch(() => {});

    await updateWindowHeight().catch(() => {});
  }

  async function restoreQuickPanelBounds() {
    const savedWidth = state.appState.settings.quickpanel_width;
    const savedHeight = state.appState.settings.quickpanel_height;
    const savedX = state.appState.settings.quickpanel_x;
    const savedY = state.appState.settings.quickpanel_y;
    const { minWidth, minHeight } = await getMinimumWindowSizePhysical();

    if (!savedWidth || !savedHeight) {
      return;
    }

    const monitors = await availableMonitors().catch(() => []);
    const fallbackMonitor =
      (await primaryMonitor().catch(() => null)) ?? monitors[0] ?? null;

    if (!fallbackMonitor) {
      await currentWindow
        .setSize(
          new PhysicalSize(
            Math.max(savedWidth, minWidth),
            Math.max(savedHeight, minHeight)
          )
        )
        .catch(() => {});

      if (savedX !== null && savedY !== null) {
        await currentWindow
          .setPosition(new PhysicalPosition(savedX, savedY))
          .catch(() => {});
      }

      return;
    }

    const targetMonitor =
      monitors.find((monitor) => {
        if (savedX === null || savedY === null) {
          return false;
        }

        const area = monitor.workArea;
        return (
          savedX >= area.position.x &&
          savedY >= area.position.y &&
          savedX < area.position.x + area.size.width &&
          savedY < area.position.y + area.size.height
        );
      }) ?? fallbackMonitor;

    const area = targetMonitor.workArea;
    const width = clamp(savedWidth, minWidth, area.size.width);
    const height = clamp(savedHeight, minHeight, area.size.height);
    const x = clamp(
      savedX ?? area.position.x + 32,
      area.position.x,
      area.position.x + Math.max(area.size.width - width, 0)
    );
    const y = clamp(
      savedY ?? area.position.y + 32,
      area.position.y,
      area.position.y + Math.max(area.size.height - height, 0)
    );

    await currentWindow.setSize(new PhysicalSize(width, height)).catch(() => {});
    await currentWindow
      .setPosition(new PhysicalPosition(x, y))
      .catch(() => {});
  }

  async function startWindowDrag() {
    await currentWindow.startDragging().catch(() => {});
  }

  async function hideWindow() {
    await currentWindow.hide().catch(() => {});
  }

  async function openLogFile() {
    await quickPanelBridge.openLogFile();
  }

  return {
    updateWindowHeight,
    applyQuickpanelModeLayout,
    restoreQuickPanelBounds,
    startWindowDrag,
    hideWindow,
    openLogFile,
  };
}
