import type { Page } from "@playwright/test";

type MockProjectState = {
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
    project_sort_mode: "manual" | "alphabetical" | "recent";
    quickpanel_mode: "normal" | "compact";
    project_manual_order: string[];
    project_recent_usage: Record<string, number>;
    timesheet_rounding_enabled: boolean;
    project_colors: Record<string, string>;
    project_groups: Record<string, string>;
    group_projects_enabled: boolean;
  };
};

type MockPreviewRequest = {
  range: "today" | "week" | "all";
  format: "full" | "recent";
};

type MockPreview = {
  title: string;
  generated_at: string;
  generated_at_epoch_ms: number;
  sheets: Array<{
    name: string;
    columns: string[];
    rows: Array<{
      label: string;
      values: number[];
      total: number;
      is_comment: boolean;
      is_total: boolean;
    }>;
  }>;
};

type MockOptions = {
  currentWindowLabel?: string;
  initialPreviewRequest?: MockPreviewRequest | null;
  previewResponse?: MockPreview;
};

const sampleProjects = [
  "Alpha Project",
  "Jot",
  "Omega / Final",
  "Project 007",
  "QP Regression: Close Button",
  "Sous Chef",
  "Test 5",
  "UX – Very Long Project Name For Overflow Checks",
  "_Internal Tools",
  "zebra-end",
  "Ægir Analytics",
  "Ångström Ops",
];

const defaultState: MockProjectState = {
  app_version: "2.1.0",
  active_project: "",
  active_comment: "",
  projects: sampleProjects,
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
    project_manual_order: sampleProjects,
    project_recent_usage: {},
    timesheet_rounding_enabled: false,
    project_colors: {},
    project_groups: {},
    group_projects_enabled: false,
  },
};

type InitialMockState = Partial<Omit<MockProjectState, "settings">> & {
  settings?: Partial<MockProjectState["settings"]>;
};

export async function installTauriMocks(
  page: Page,
  initialState?: InitialMockState,
  options?: MockOptions
) {
  const mergedState: MockProjectState = {
    ...defaultState,
    ...initialState,
    projects: initialState?.projects ?? defaultState.projects,
    adhoc_projects: initialState?.adhoc_projects ?? defaultState.adhoc_projects,
    settings: {
      ...defaultState.settings,
      ...initialState?.settings,
      project_manual_order:
        initialState?.settings?.project_manual_order ?? initialState?.projects ?? defaultState.settings.project_manual_order,
      project_recent_usage:
        initialState?.settings?.project_recent_usage ?? defaultState.settings.project_recent_usage,
    },
  };

  const previewResponse: MockPreview = options?.previewResponse ?? {
    title: "Full timesheet",
    generated_at: "2026-04-30 07:09",
    generated_at_epoch_ms: Date.UTC(2026, 3, 30, 5, 9, 0),
    sheets: [
      {
        name: "2026-18",
        columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        rows: [
          {
            label: "Alpha",
            values: [1, 0, 0, 0, 0, 0, 0],
            total: 1,
            is_comment: false,
            is_total: false,
          },
          {
            label: "Total",
            values: [1, 0, 0, 0, 0, 0, 0],
            total: 1,
            is_comment: false,
            is_total: true,
          },
        ],
      },
    ],
  };

  await page.addInitScript(({
    state,
    currentWindowLabel,
    initialPreviewRequest,
    initialPreviewResponse,
  }: {
    state: MockProjectState;
    currentWindowLabel: string;
    initialPreviewRequest: MockPreviewRequest | null;
    initialPreviewResponse: MockPreview;
  }) => {
    const params = new URLSearchParams(window.location.search);
    const queryWindowLabel = params.get("mockWindowLabel");
    const queryPreviewRange = params.get("mockPreviewRange") as MockPreviewRequest["range"] | null;
    const queryPreviewFormat = params.get("mockPreviewFormat") as MockPreviewRequest["format"] | null;
    const queryPreviewScenario = params.get("mockPreviewScenario");

    if (queryWindowLabel) {
      currentWindowLabel = queryWindowLabel;
    }

    if (queryPreviewRange && queryPreviewFormat) {
      initialPreviewRequest = {
        range: queryPreviewRange,
        format: queryPreviewFormat,
      };

      initialPreviewResponse =
        queryPreviewScenario === "banding"
          ? {
              title: "Full timesheet",
              generated_at: "2026-04-30 07:09",
              generated_at_epoch_ms: Date.UTC(2026, 3, 30, 5, 9, 0),
              sheets: [
                {
                  name: "2026-18",
                  columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                  rows: [
                    {
                      label: "Alpha",
                      values: [1, 0, 0, 0, 0, 0, 0],
                      total: 1,
                      is_comment: false,
                      is_total: false,
                    },
                    {
                      label: "  - Prep",
                      values: [1, 0, 0, 0, 0, 0, 0],
                      total: 1,
                      is_comment: true,
                      is_total: false,
                    },
                    {
                      label: "Beta",
                      values: [0, 2, 0, 0, 0, 0, 0],
                      total: 2,
                      is_comment: false,
                      is_total: false,
                    },
                    {
                      label: "Total",
                      values: [1, 2, 0, 0, 0, 0, 0],
                      total: 3,
                      is_comment: false,
                      is_total: true,
                    },
                  ],
                },
              ],
            }
          : {
              title: queryPreviewFormat === "recent" ? "Yesterday + today overview" : "Full timesheet",
              generated_at: "2026-04-30 07:09",
              generated_at_epoch_ms: Date.UTC(2026, 3, 30, 5, 9, 0),
              sheets: [
                {
                  name: "2026-18",
                  columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                  rows: [
                    {
                      label: "Alpha",
                      values: [1, 0, 0, 0, 0, 0, 0],
                      total: 1,
                      is_comment: false,
                      is_total: false,
                    },
                    {
                      label: "Total",
                      values: [1, 0, 0, 0, 0, 0, 0],
                      total: 1,
                      is_comment: false,
                      is_total: true,
                    },
                  ],
                },
              ],
            };
    }

    const baseState = JSON.parse(JSON.stringify(state)) as MockProjectState;

    const setupMocks = (
      nextState: Partial<MockProjectState> | undefined,
      nextWindowLabel: string,
      nextPreviewRequest: MockPreviewRequest | null,
      nextPreviewResponse: MockPreview
    ) => {
      const clonedState = JSON.parse(
        JSON.stringify({
          ...baseState,
          ...nextState,
          settings: {
            ...baseState.settings,
            ...nextState?.settings,
            project_manual_order:
              nextState?.settings?.project_manual_order ?? baseState.settings.project_manual_order,
            project_recent_usage:
              nextState?.settings?.project_recent_usage ?? baseState.settings.project_recent_usage,
          },
        })
      ) as MockProjectState;
      const callbacks = new Map<number, (payload: unknown) => void>();
      const eventHandlers = new Map<string, number[]>();
      const invokedCommands: string[] = [];
      let callbackId = 1;
      let eventId = 1;
      let previewRequest = nextPreviewRequest;
      const previewResponse = nextPreviewResponse;
      let recentCounter = Math.max(
        0,
        ...Object.values(clonedState.settings.project_recent_usage || {})
      );

      function emitEvent(event: string, payload?: unknown) {
        const handlers = eventHandlers.get(event) || [];
        for (const id of handlers) {
          const callback = callbacks.get(id);
          callback?.({ event, id: eventId++, payload });
        }
      }

      function nextRecentStamp() {
        recentCounter += 1;
        return recentCounter;
      }

      function handleSelectProject(project: string) {
        if (clonedState.active_project === project) {
          clonedState.active_project = "";
        } else {
          clonedState.active_project = project;
          clonedState.settings.project_recent_usage[project] = nextRecentStamp();
        }
        clonedState.active_comment = "";
      }

      (window as Window & {
        __TAURI_INTERNALS__?: Record<string, unknown>;
        __TAURI_EVENT_PLUGIN_INTERNALS__?: Record<string, unknown>;
        __TAURI_MOCK__?: Record<string, unknown>;
      }).__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: nextWindowLabel },
          currentWebview: { label: nextWindowLabel, windowLabel: nextWindowLabel },
        },
        convertFileSrc: (path: string) => path,
        transformCallback: (fn: (payload: unknown) => void) => {
          const id = callbackId++;
          callbacks.set(id, fn);
          return id;
        },
        unregisterCallback: (id: number) => {
          callbacks.delete(id);
        },
        runCallback: (id: number, payload: unknown) => {
          callbacks.get(id)?.(payload);
        },
        callbacks,
        invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
          invokedCommands.push(cmd);
          switch (cmd) {
          case "get_state":
            return JSON.parse(JSON.stringify(clonedState));
          case "get_timesheet_preview_bootstrap":
            return {
              request: previewRequest,
              rounding_enabled: clonedState.settings.timesheet_rounding_enabled,
            };
          case "preview_timesheet":
            return JSON.parse(JSON.stringify(previewResponse));
          case "select_project":
            handleSelectProject(String(args.project ?? ""));
            return null;
          case "add_project": {
            const value = String(args.value ?? "").trim();
            if (value && !clonedState.projects.includes(value)) {
              clonedState.projects.push(value);
              clonedState.settings.project_manual_order.push(value);
            }
            return null;
          }
          case "quick_project": {
            const value = String(args.value ?? "").trim();
            if (value) {
              if (!clonedState.projects.includes(value) && !clonedState.adhoc_projects.includes(value)) {
                clonedState.adhoc_projects.push(value);
              }
              clonedState.active_project = value;
              clonedState.active_comment = "";
              clonedState.settings.project_recent_usage[value] = nextRecentStamp();
            }
            return null;
          }
          case "set_comment":
            clonedState.active_comment = String(args.value ?? "");
            return null;
          case "remove_project": {
            const project = String(args.project ?? "");
            clonedState.projects = clonedState.projects.filter((p) => p !== project);
            clonedState.adhoc_projects = clonedState.adhoc_projects.filter((p) => p !== project);
            clonedState.settings.project_manual_order =
              clonedState.settings.project_manual_order.filter((p) => p !== project);
            delete clonedState.settings.project_recent_usage[project];
            if (clonedState.active_project === project) {
              clonedState.active_project = "";
              clonedState.active_comment = "";
            }
            return null;
          }
          case "save_ui_settings":
            clonedState.settings.always_on_top = Boolean(args.alwaysOnTop);
            clonedState.settings.open_on_start = Boolean(args.openOnStart);
            clonedState.settings.quickpanel_opacity = Number(args.quickpanelOpacity);
            clonedState.settings.project_sort_mode = String(args.projectSortMode) as MockProjectState["settings"]["project_sort_mode"];
            clonedState.settings.quickpanel_mode = String(args.quickpanelMode) as MockProjectState["settings"]["quickpanel_mode"];
            clonedState.settings.project_manual_order = [...((args.projectManualOrder as string[]) ?? [])];
            clonedState.settings.project_recent_usage = {
              ...((args.projectRecentUsage as Record<string, number>) ?? {}),
            };
            clonedState.settings.timesheet_rounding_enabled = Boolean(args.timesheetRoundingEnabled);
            clonedState.settings.project_colors = {
              ...((args.projectColors as Record<string, string>) ?? {}),
            };
            clonedState.settings.project_groups = {
              ...((args.projectGroups as Record<string, string>) ?? {}),
            };
            clonedState.settings.group_projects_enabled = Boolean(args.groupProjectsEnabled);
            return null;
          case "set_timesheet_rounding_enabled":
            clonedState.settings.timesheet_rounding_enabled = Boolean(args.enabled);
            return null;
          case "set_project_color": {
            const project = String(args.project ?? "");
            if (args.color === null || args.color === undefined) {
              delete clonedState.settings.project_colors[project];
            } else {
              clonedState.settings.project_colors[project] = String(args.color);
            }
            return null;
          }
          case "set_project_group": {
            const project = String(args.project ?? "");
            if (args.group === null || args.group === undefined) {
              delete clonedState.settings.project_groups[project];
            } else {
              clonedState.settings.project_groups[project] = String(args.group);
            }
            return null;
          }
          case "save_quickpanel_bounds":
          case "set_update_available":
          case "log_from_frontend":
          case "hide_timesheet_preview_window":
          case "generate_timesheet_export":
          case "reset_timesheet":
          case "reset_projects":
          case "open_log_file":
          case "open_diagnostic_log":
          case "open_feedback":
          case "open_github_issues":
          case "open_portfolio":
          case "open_project_homepage":
          case "open_release_notes":
            return null;
          case "open_timesheet_preview_window":
            previewRequest = {
              range: String(args.range ?? "all") as MockPreviewRequest["range"],
              format: String(args.format ?? "full") as MockPreviewRequest["format"],
            };
            emitEvent("show-timesheet-preview", previewRequest);
            return null;
          case "plugin:updater|check":
            return null;
          case "plugin:event|listen": {
            const event = String(args.event ?? "");
            const handler = Number(args.handler);
            const list = eventHandlers.get(event) ?? [];
            list.push(handler);
            eventHandlers.set(event, list);
            return handler;
          }
          case "plugin:event|unlisten": {
            const event = String(args.event ?? "");
            const id = Number(args.eventId);
            const list = eventHandlers.get(event) ?? [];
            eventHandlers.set(
              event,
              list.filter((handlerId) => handlerId !== id)
            );
            return null;
          }
          case "plugin:event|emit":
            emitEvent(String(args.event ?? ""), args.payload);
            return null;
          case "plugin:window|set_size_constraints":
          case "plugin:window|set_size":
          case "plugin:window|set_position":
          case "plugin:window|set_always_on_top":
          case "plugin:window|show":
          case "plugin:window|hide":
          case "plugin:window|set_focus":
          case "plugin:window|close":
          case "plugin:window|start_dragging":
            return null;
          case "plugin:window|outer_size":
            return { width: 430, height: 650 };
          case "plugin:window|scale_factor":
            return 1;
          case "plugin:window|outer_position":
            return { x: 100, y: 100 };
          case "plugin:window|available_monitors":
            return [];
          case "plugin:window|primary_monitor":
            return null;
            default:
              return null;
          }
        },
      };

      (window as Window & {
        __TAURI_EVENT_PLUGIN_INTERNALS__?: Record<string, unknown>;
        __TAURI_MOCK__?: Record<string, unknown>;
      }).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: (event: string, id: number) => {
          const list = eventHandlers.get(event) ?? [];
          eventHandlers.set(
            event,
            list.filter((handlerId) => handlerId !== id)
          );
        },
      };

      (window as Window & {
        __TAURI_MOCK__?: Record<string, unknown>;
      }).__TAURI_MOCK__ = {
        invokedCommands,
        setupMocks,
      };
    };

    setupMocks(state, currentWindowLabel, initialPreviewRequest, initialPreviewResponse);
  }, {
    state: mergedState,
    currentWindowLabel: options?.currentWindowLabel ?? "main",
    initialPreviewRequest: options?.initialPreviewRequest ?? null,
    initialPreviewResponse: previewResponse,
  });
}
