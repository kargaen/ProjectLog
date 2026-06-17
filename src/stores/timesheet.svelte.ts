import type { TimesheetFormat, TimesheetPreview, TimesheetRange } from "../models/types";

export type TimesheetState = {
  preview: TimesheetPreview | null;
  range: TimesheetRange;
  format: TimesheetFormat;
  sheetIndex: number;
  roundingEnabled: boolean;
  loading: boolean;
  refreshing: boolean;
  hoveredRowIndex: number | null;
  hoveredColumnIndex: number | null;
};

let state = $state<TimesheetState>({
  preview: null,
  range: "all",
  format: "full",
  sheetIndex: 0,
  roundingEnabled: false,
  loading: true,
  refreshing: false,
  hoveredRowIndex: null,
  hoveredColumnIndex: null,
});

export function getTimesheetState(): TimesheetState {
  return state;
}

export function patchTimesheetState(patch: Partial<TimesheetState>): void {
  Object.assign(state, patch);
}
