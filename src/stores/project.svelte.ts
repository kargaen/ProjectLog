import type { ProjectState } from "../models/types";

function defaultProjectState(): ProjectState {
  return {
    app_version: "",
    active_project: "",
    active_comment: "",
    projects: [],
    adhoc_projects: [],
    update_available: false,
    settings: {
      always_on_top: false,
      open_on_start: false,
      quickpanel_x: null,
      quickpanel_y: null,
      quickpanel_width: null,
      quickpanel_height: null,
      quickpanel_opacity: 1,
      project_sort_mode: "manual",
      quickpanel_mode: "normal",
      project_manual_order: [],
      project_recent_usage: {},
      timesheet_rounding_enabled: false,
    },
  };
}

let state = $state<ProjectState>(defaultProjectState());

export function getProjectState(): ProjectState {
  return state;
}

export function setProjectState(next: ProjectState): void {
  state = next;
}
