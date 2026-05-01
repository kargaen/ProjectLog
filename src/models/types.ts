export type SortMode = "manual" | "alphabetical" | "recent";
export type QuickPanelMode = "normal" | "compact";
export type TimesheetRange = "today" | "week" | "all";
export type TimesheetFormat = "full" | "recent";

export type ProjectState = {
  app_version: string;
  active_project: string;
  active_comment: string;
  projects: string[];
  adhoc_projects: string[];
  update_available: boolean;
  settings: {
    always_on_top: boolean;
    open_on_start: boolean;
    quickpanel_x: number | null;
    quickpanel_y: number | null;
    quickpanel_width: number | null;
    quickpanel_height: number | null;
    quickpanel_opacity: number;
    project_sort_mode: SortMode;
    quickpanel_mode: QuickPanelMode;
    project_manual_order: string[];
    project_recent_usage: Record<string, number>;
    timesheet_rounding_enabled: boolean;
  };
};

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
};
