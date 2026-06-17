import { invoke } from "@tauri-apps/api/core";

import type { ProjectState } from "../../models/types";

export function createProjectBridge() {
  return {
    getState() {
      return invoke<ProjectState>("get_state");
    },
    selectProject(project: string) {
      return invoke("select_project", { project });
    },
    addProject(value: string) {
      return invoke("add_project", { value });
    },
    quickProject(value: string) {
      return invoke("quick_project", { value });
    },
    setComment(value: string) {
      return invoke("set_comment", { value });
    },
    removeProject(project: string) {
      return invoke("remove_project", { project });
    },
    submitInput(mode: string, value: string) {
      return invoke("submit_input", { mode, value });
    },
    resetProjects() {
      return invoke("reset_projects");
    },
  };
}

export type ProjectBridge = ReturnType<typeof createProjectBridge>;
