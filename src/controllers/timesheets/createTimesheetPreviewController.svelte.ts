import { getCurrentWindow } from "@tauri-apps/api/window";

import { createLogger } from "../../lib/logger";
import {
  buildTimesheetDisplayRows,
  formatGeneratedAge,
  formatGeneratedTimestamp,
  formatPreviewHours,
} from "../../lib/timesheet-preview";
import type {
  TimesheetFormat,
  TimesheetPreview,
  TimesheetPreviewRequest,
  TimesheetRange,
} from "../../models/types";
import {
  createSettingsBridge,
  type SettingsBridge,
} from "../../services/bridge/settingsBridge";
import {
  createTimesheetBridge,
  type TimesheetBridge,
} from "../../services/bridge/timesheetBridge";

const log = createLogger("timesheet-preview");

type CreateTimesheetPreviewControllerDeps = {
  timesheetBridge?: TimesheetBridge;
  settingsBridge?: SettingsBridge;
};

export function createTimesheetPreviewController(
  deps: CreateTimesheetPreviewControllerDeps = {}
) {
  const timesheetBridge =
    deps.timesheetBridge ?? createTimesheetBridge();
  const settingsBridge =
    deps.settingsBridge ?? createSettingsBridge();
  const currentWindow = getCurrentWindow();

  const SCALE_STEP = 0.05;
  const SCALE_MIN = 0.5;
  const SCALE_MAX = 2.0;

  const state = $state({
    timesheetPreview: null as TimesheetPreview | null,
    timesheetPreviewSheetIndex: 0,
    timesheetRoundingEnabled: false,
    loading: true,
    refreshing: false,
    hoveredRowIndex: null as number | null,
    hoveredColumnIndex: null as number | null,
  });

  let uiFontScale = $state(1);
  let fontScaleIndicator = $state({ visible: false, scale: 1 });
  let fontScaleIndicatorTimer: ReturnType<typeof setTimeout> | undefined;

  let timesheetPreviewRange = $state<TimesheetRange>("all");
  let timesheetPreviewFormat = $state<TimesheetFormat>("full");
  let relativeTimeNow = $state(Date.now());

  const view = {
    get displayedTimesheetSheet() {
      return state.timesheetPreview
        ? state.timesheetPreview.sheets[state.timesheetPreviewSheetIndex]
        : null;
    },
    get timesheetPreviewYears() {
      if (!state.timesheetPreview) {
        return [];
      }

      const years = new Set(
        state.timesheetPreview.sheets
          .map((sheet) => sheet.name.split("-")[0])
          .filter((value) => /^\d{4}$/.test(value))
      );

      return [...years].sort();
    },
    get displayedTimesheetRows() {
      return buildTimesheetDisplayRows(
        this.displayedTimesheetSheet,
        state.timesheetRoundingEnabled
      );
    },
    get generatedStatus() {
      if (!state.timesheetPreview) {
        return "";
      }

      return `Generated at ${formatGeneratedTimestamp(
        state.timesheetPreview.generated_at_epoch_ms
      )}, ${formatGeneratedAge(
        state.timesheetPreview.generated_at_epoch_ms,
        relativeTimeNow
      )}`;
    },
  };

  function formatHours(value: number) {
    return formatPreviewHours(value, state.timesheetRoundingEnabled);
  }

  async function startWindowDrag() {
    await currentWindow.startDragging().catch(() => {});
  }

  function clearCrosshair() {
    state.hoveredRowIndex = null;
    state.hoveredColumnIndex = null;
  }

  function setHoveredCell(rowIndex: number, columnIndex: number) {
    state.hoveredRowIndex = rowIndex;
    state.hoveredColumnIndex = columnIndex;
  }

  function selectSheet(index: number) {
    state.timesheetPreviewSheetIndex = index;
    clearCrosshair();
  }

  function updateSheetIndex(
    nextPreview: TimesheetPreview,
    format: TimesheetFormat
  ) {
    const currentSheetName = view.displayedTimesheetSheet?.name;
    if (format !== "full") {
      state.timesheetPreviewSheetIndex = 0;
      return;
    }

    const matchingIndex = nextPreview.sheets.findIndex(
      (sheet) => sheet.name === currentSheetName
    );

    state.timesheetPreviewSheetIndex =
      matchingIndex >= 0
        ? matchingIndex
        : Math.max(nextPreview.sheets.length - 1, 0);
  }

  async function loadPreview(
    range: TimesheetRange,
    format: TimesheetFormat,
    options?: { preservePreview?: boolean }
  ) {
    const preservePreview = options?.preservePreview ?? false;
    timesheetPreviewRange = range;
    timesheetPreviewFormat = format;

    if (!preservePreview) {
      state.timesheetPreview = null;
      state.loading = true;
    } else {
      state.refreshing = true;
    }

    try {
      const preview = await timesheetBridge.previewTimesheet(range, format);
      state.timesheetPreview = preview;
      updateSheetIndex(preview, format);
      clearCrosshair();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error);
      log.warn("loadPreview failed", { range, format, message });
      alert(message);
    } finally {
      state.loading = false;
      state.refreshing = false;
    }
  }

  async function refreshNow() {
    await loadPreview(timesheetPreviewRange, timesheetPreviewFormat, {
      preservePreview: Boolean(state.timesheetPreview),
    });
  }

  async function closeTimesheetPreviewWindow() {
    await timesheetBridge.hideTimesheetPreviewWindow().catch(() => {});
  }

  async function toggleTimesheetRounding() {
    const next = !state.timesheetRoundingEnabled;
    state.timesheetRoundingEnabled = next;
    await settingsBridge.setTimesheetRoundingEnabled(next);
  }

  async function exportTimesheet() {
    try {
      await timesheetBridge.generateTimesheetExport(
        timesheetPreviewRange,
        timesheetPreviewFormat
      );
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error);
      log.warn("exportTimesheet failed", { message });
      alert(message);
    }
  }

  function mount() {
    log.info("mounted");

    const timer = setInterval(() => {
      relativeTimeNow = Date.now();
    }, 1000);

    let disposed = false;
    let cleanup = () => {
      clearInterval(timer);
    };

    void (async () => {
      const unlistenTimesheetPreview =
        currentWindow.listen<TimesheetPreviewRequest>(
          "show-timesheet-preview",
          (event) => {
            void loadPreview(event.payload.range, event.payload.format, {
              preservePreview: Boolean(state.timesheetPreview),
            });
          }
        );

      cleanup = () => {
        clearInterval(timer);
        void unlistenTimesheetPreview.then((fn) => fn());
      };

      function handleWheel(event: WheelEvent) {
        if (!event.ctrlKey) return;
        event.preventDefault();
        const delta = event.deltaY < 0 ? SCALE_STEP : -SCALE_STEP;
        const next = Math.round(
          Math.min(SCALE_MAX, Math.max(SCALE_MIN, uiFontScale + delta)) * 100
        ) / 100;
        uiFontScale = next;
        document.documentElement.style.setProperty("--font-scale", String(next));
        fontScaleIndicator = { visible: true, scale: next };
        clearTimeout(fontScaleIndicatorTimer);
        fontScaleIndicatorTimer = setTimeout(() => {
          fontScaleIndicator = { ...fontScaleIndicator, visible: false };
        }, 1200);
        void settingsBridge.setUiFontScale(next).catch(() => {});
      }

      window.addEventListener("wheel", handleWheel, { passive: false });

      cleanup = () => {
        clearInterval(timer);
        window.removeEventListener("wheel", handleWheel);
        void unlistenTimesheetPreview.then((fn) => fn());
      };

      try {
        const bootstrap = await timesheetBridge.getPreviewBootstrap();
        state.timesheetRoundingEnabled = bootstrap.rounding_enabled;
        uiFontScale = bootstrap.ui_font_scale ?? 1;
        document.documentElement.style.setProperty("--font-scale", String(uiFontScale));

        if (bootstrap.request) {
          await loadPreview(
            bootstrap.request.range,
            bootstrap.request.format
          );
        } else {
          state.loading = false;
        }
      } catch (error) {
        state.loading = false;
        const message =
          error instanceof Error ? error.message : String(error);
        log.warn("bootstrap failed", { message });
        alert(message);
      }

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
    state,
    view,
    get uiFontScale() { return uiFontScale; },
    get fontScaleIndicator() { return fontScaleIndicator; },
    formatHours,
    mount,
    startWindowDrag,
    clearCrosshair,
    setHoveredCell,
    selectSheet,
    refreshNow,
    closeTimesheetPreviewWindow,
    toggleTimesheetRounding,
    exportTimesheet,
  };
}
