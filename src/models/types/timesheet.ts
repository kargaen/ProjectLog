export type TimesheetRange = "today" | "week" | "all";
export type TimesheetFormat = "full" | "recent";

export type TimesheetPreviewRow = {
  label: string;
  values: number[];
  total: number;
  is_comment: boolean;
  is_total: boolean;
};

export type TimesheetPreviewSheet = {
  name: string;
  columns: string[];
  rows: TimesheetPreviewRow[];
};

export type TimesheetPreview = {
  title: string;
  generated_at: string;
  generated_at_epoch_ms: number;
  sheets: TimesheetPreviewSheet[];
};

export type TimesheetPreviewRequest = {
  range: TimesheetRange;
  format: TimesheetFormat;
};

export type TimesheetPreviewBootstrap = {
  request: TimesheetPreviewRequest | null;
  rounding_enabled: boolean;
  ui_font_scale: number;
};
