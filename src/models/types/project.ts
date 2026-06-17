import type { QuickPanelMode, SortMode } from "./settings";

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
