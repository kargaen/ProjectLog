import { invoke } from "@tauri-apps/api/core";

import type {
  TimesheetFormat,
  TimesheetPreview,
  TimesheetPreviewBootstrap,
  TimesheetRange,
} from "../../models/types";

export function createTimesheetBridge() {
  return {
    getPreviewBootstrap() {
      return invoke<TimesheetPreviewBootstrap>("get_timesheet_preview_bootstrap");
    },
    previewTimesheet(range: TimesheetRange, format: TimesheetFormat) {
      return invoke<TimesheetPreview>("preview_timesheet", { range, format });
    },
    openTimesheetPreviewWindow(
      range: TimesheetRange,
      format: TimesheetFormat = "full"
    ) {
      return invoke("open_timesheet_preview_window", { range, format });
    },
    hideTimesheetPreviewWindow() {
      return invoke("hide_timesheet_preview_window");
    },
    setTimesheetRoundingEnabled(enabled: boolean) {
      return invoke("set_timesheet_rounding_enabled", { enabled });
    },
    generateTimesheetExport(
      range: TimesheetRange,
      format: TimesheetFormat = "full"
    ) {
      return invoke("generate_timesheet_export", { range, format });
    },
    resetTimesheet() {
      return invoke("reset_timesheet");
    },
  };
}

export type TimesheetBridge = ReturnType<typeof createTimesheetBridge>;
