import { invoke } from "@tauri-apps/api/core";

export function createShellBridge() {
  return {
    setUpdateAvailable(available: boolean) {
      return invoke("set_update_available", { available });
    },
    openLogFile() {
      return invoke("open_log_file");
    },
    openDiagnosticLog() {
      return invoke("open_diagnostic_log");
    },
    openGithubIssues() {
      return invoke("open_github_issues");
    },
    openProjectHomepage() {
      return invoke("open_project_homepage");
    },
    openPortfolio() {
      return invoke("open_portfolio");
    },
    openReleaseNotes() {
      return invoke("open_release_notes");
    },
  };
}

export type ShellBridge = ReturnType<typeof createShellBridge>;
