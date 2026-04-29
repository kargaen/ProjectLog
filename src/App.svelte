<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import {
    availableMonitors,
    getCurrentWindow,
    LogicalPosition,
    LogicalSize,
    primaryMonitor,
  } from "@tauri-apps/api/window";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { createLogger } from "./lib/logger";
  import appIcon from "../icon.svg";

  const log = createLogger("quickpanel");
  const MIN_QUICKPANEL_WIDTH = 300;
  const MIN_NORMAL_HEIGHT = 430;
  const MIN_COMPACT_HEIGHT = 220;
  const MIN_OPACITY = 0.35;

  type SortMode = "manual" | "alphabetical" | "recent";
  type QuickPanelMode = "normal" | "compact";
  type TimesheetRange = "today" | "week" | "all";
  type TimesheetFormat = "full" | "recent";

  type ProjectState = {
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

  type TimesheetPreviewRow = {
    label: string;
    values: number[];
    total: number;
    is_comment: boolean;
    is_total: boolean;
  };

  type TimesheetPreviewSheet = {
    name: string;
    columns: string[];
    rows: TimesheetPreviewRow[];
  };

  type TimesheetPreview = {
    title: string;
    sheets: TimesheetPreviewSheet[];
  };

  let appState = $state<ProjectState>({
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
  });
  let quickName = $state("");
  let newProjectName = $state("");
  let commentText = $state("");
  let alwaysOnTop = $state(false);
  let openOnStart = $state(false);
  let pendingUpdate = $state<Update | null>(null);
  let updateStatus = $state<"idle" | "available" | "downloading" | "ready">("idle");
  let updateVersion = $state("");
  let updateProgress = $state(0);
  let updatePromptOpen = $state(false);
  let quickPanelOpacity = $state(1);
  let sortMode = $state<SortMode>("manual");
  let quickPanelMode = $state<QuickPanelMode>("normal");
  let recentProjects = $state<Record<string, number>>({});
  let manualOrder = $state<string[]>([]);
  let draggedProject = $state<string | null>(null);
  let dropTargetProject = $state<string | null>(null);
  let dropPosition = $state<"before" | "after">("before");
  let effectiveSortMode = $derived<SortMode>(quickPanelMode === "compact" ? "manual" : sortMode);

  let dialogOpen = $state(false);
  let aboutOpen = $state(false);
  let mode = $state("");
  let title = $state("");
  let value = $state("");
  let inputEl: HTMLInputElement | undefined = $state();
  let settingsSaveTimer: ReturnType<typeof setTimeout> | undefined = $state();
  let closeInputOnSubmit = $state(true);
  let ignoredStateChangedEvents = $state(0);
  const windowParams = new URLSearchParams(window.location.search);
  const isTimesheetPreviewWindow = windowParams.get("window") === "timesheet-preview";
  let timesheetPreview = $state<TimesheetPreview | null>(null);
  let timesheetPreviewRange = $state<TimesheetRange>("all");
  let timesheetPreviewFormat = $state<TimesheetFormat>("full");
  let timesheetPreviewSheetIndex = $state(0);
  let timesheetRoundingEnabled = $state(false);

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  function syncManualOrder(projects: string[]) {
    const seen = new Set(projects);
    const ordered = manualOrder.filter((project) => seen.has(project));
    for (const project of projects) {
      if (!ordered.includes(project)) ordered.push(project);
    }
    if (
      ordered.length !== manualOrder.length ||
      ordered.some((project, index) => project !== manualOrder[index])
    ) {
      manualOrder = ordered;
      queueSettingsSave();
    }
  }

  function isEnterSubmit(event: KeyboardEvent) {
    return event.key === "Enter" && !event.shiftKey;
  }

  function roundHalfPreservingSum(values: number[]) {
    const step = 0.5;
    const floors = values.map((value) => Math.floor(value / step) * step);
    const roundedTotal = Math.round((values.reduce((sum, value) => sum + value, 0) / step)) * step;
    const currentTotal = floors.reduce((sum, value) => sum + value, 0);
    let increments = Math.round((roundedTotal - currentTotal) / step);
    const ranked = values
      .map((value, index) => ({ index, remainder: value - floors[index] }))
      .sort((a, b) => b.remainder - a.remainder || a.index - b.index);
    const result = [...floors];
    for (let i = 0; i < ranked.length && increments > 0; i += 1) {
      result[ranked[i].index] += step;
      increments -= 1;
    }
    return result;
  }

  function formatHours(value: number) {
    return timesheetRoundingEnabled ? value.toFixed(1) : value.toFixed(2);
  }

  async function startWindowDrag() {
    await getCurrentWindow().startDragging().catch(() => {});
  }

  async function restoreQuickPanelBounds() {
    const win = getCurrentWindow();
    const savedWidth = appState.settings.quickpanel_width;
    const savedHeight = appState.settings.quickpanel_height;
    const savedX = appState.settings.quickpanel_x;
    const savedY = appState.settings.quickpanel_y;
    const minHeight = appState.settings.quickpanel_mode === "compact" ? MIN_COMPACT_HEIGHT : MIN_NORMAL_HEIGHT;

    if (!savedWidth || !savedHeight) {
      return;
    }

    const monitors = await availableMonitors().catch(() => []);
    const fallbackMonitor = (await primaryMonitor().catch(() => null)) ?? monitors[0] ?? null;

    if (!fallbackMonitor) {
      await win.setSize(new LogicalSize(savedWidth, savedHeight)).catch(() => {});
      if (savedX !== null && savedY !== null) {
        await win.setPosition(new LogicalPosition(savedX, savedY)).catch(() => {});
      }
      return;
    }

    const targetMonitor =
      monitors.find((monitor) => {
        if (savedX === null || savedY === null) return false;
        const area = monitor.workArea;
        return (
          savedX >= area.position.x &&
          savedY >= area.position.y &&
          savedX < area.position.x + area.size.width &&
          savedY < area.position.y + area.size.height
        );
      }) ?? fallbackMonitor;

    const area = targetMonitor.workArea;
    const width = clamp(savedWidth, MIN_QUICKPANEL_WIDTH, area.size.width);
    const height = clamp(savedHeight, minHeight, area.size.height);
    const x = clamp(
      savedX ?? area.position.x + 32,
      area.position.x,
      area.position.x + Math.max(area.size.width - width, 0)
    );
    const y = clamp(
      savedY ?? area.position.y + 32,
      area.position.y,
      area.position.y + Math.max(area.size.height - height, 0)
    );

    await win.setSize(new LogicalSize(width, height)).catch(() => {});
    await win.setPosition(new LogicalPosition(x, y)).catch(() => {});
  }

  function applyAppState(nextState: ProjectState, options?: { preserveMode?: boolean }) {
    const preserveMode = options?.preserveMode ?? false;
    const currentQuickPanelMode = quickPanelMode;

    appState.app_version = nextState.app_version;
    appState.active_project = nextState.active_project;
    appState.active_comment = nextState.active_comment;
    appState.projects = nextState.projects;
    appState.adhoc_projects = nextState.adhoc_projects;
    appState.update_available = nextState.update_available;

    appState.settings.always_on_top = nextState.settings.always_on_top;
    appState.settings.open_on_start = nextState.settings.open_on_start;
    appState.settings.quickpanel_x = nextState.settings.quickpanel_x;
    appState.settings.quickpanel_y = nextState.settings.quickpanel_y;
    appState.settings.quickpanel_width = nextState.settings.quickpanel_width;
    appState.settings.quickpanel_height = nextState.settings.quickpanel_height;
    appState.settings.quickpanel_opacity = nextState.settings.quickpanel_opacity;
    appState.settings.project_sort_mode = nextState.settings.project_sort_mode;
    appState.settings.quickpanel_mode = nextState.settings.quickpanel_mode;
    appState.settings.project_manual_order = nextState.settings.project_manual_order;
    appState.settings.project_recent_usage = nextState.settings.project_recent_usage;
    appState.settings.timesheet_rounding_enabled = nextState.settings.timesheet_rounding_enabled;

    commentText = nextState.active_comment;
    alwaysOnTop = nextState.settings.always_on_top;
    openOnStart = nextState.settings.open_on_start;
    quickPanelOpacity = nextState.settings.quickpanel_opacity;
    sortMode = nextState.settings.project_sort_mode ?? "manual";
    quickPanelMode = preserveMode ? currentQuickPanelMode : (nextState.settings.quickpanel_mode ?? "normal");
    manualOrder = nextState.settings.project_manual_order ?? [];
    recentProjects = nextState.settings.project_recent_usage ?? {};
    timesheetRoundingEnabled = nextState.settings.timesheet_rounding_enabled ?? false;
    syncManualOrder([...nextState.projects, ...nextState.adhoc_projects]);
  }

  async function loadState(options?: { preserveMode?: boolean }) {
    log.debug("loadState");
    const nextState = await invoke<ProjectState>("get_state");
    applyAppState(nextState, options);
  }

  async function refreshFromCommand<T>(command: Promise<T>, options?: { preserveMode?: boolean }) {
    ignoredStateChangedEvents += 1;
    const result = await command;
    await loadState(options);
    return result;
  }

  async function persistUiSettings() {
    await invoke("save_ui_settings", {
      alwaysOnTop,
      openOnStart,
      quickpanelOpacity: quickPanelOpacity,
      projectSortMode: sortMode,
      quickpanelMode,
      projectManualOrder: manualOrder,
      projectRecentUsage: recentProjects,
      timesheetRoundingEnabled,
    });
  }

  async function applyQuickpanelModeLayout(mode: QuickPanelMode) {
    const win = getCurrentWindow();
    const minHeight = mode === "compact" ? MIN_COMPACT_HEIGHT : MIN_NORMAL_HEIGHT;
    await win.setMinSize(new LogicalSize(MIN_QUICKPANEL_WIDTH, minHeight)).catch(() => {});

    try {
      const size = await win.outerSize();
      if (size.height < minHeight) {
        await win.setSize(new LogicalSize(Math.max(size.width, MIN_QUICKPANEL_WIDTH), minHeight)).catch(() => {});
      }
    } catch {
    }
  }

  async function selectProject(project: string) {
    log.info("selectProject", { project });
    await refreshFromCommand(invoke("select_project", { project }), { preserveMode: true });
  }

  async function addProject() {
    const value = newProjectName.trim();
    if (!value) return;
    log.info("addProject", { length: value.length });
    await refreshFromCommand(invoke("add_project", { value }), { preserveMode: true });
    newProjectName = "";
  }

  async function trackQuick(addToo: boolean) {
    const value = quickName.trim();
    if (!value) return;
    log.info("trackQuick", { addToo, length: value.length });
    if (addToo) {
      await refreshFromCommand(invoke("add_project", { value }), { preserveMode: true });
    }
    await refreshFromCommand(invoke("quick_project", { value }), { preserveMode: true });
    quickName = "";
  }

  async function saveComment() {
    log.info("saveComment", { length: commentText.trim().length });
    await refreshFromCommand(invoke("set_comment", { value: commentText.trim() }), { preserveMode: true });
  }

  async function clearComment() {
    commentText = "";
    log.info("clearComment");
    await refreshFromCommand(invoke("set_comment", { value: "" }), { preserveMode: true });
  }

  async function removeProject(project: string) {
    log.warn("removeProject", { project });
    await refreshFromCommand(invoke("remove_project", { project }), { preserveMode: true });
  }

  async function saveAdhocProject(project: string) {
    log.info("saveAdhocProject", { project });
    await refreshFromCommand(invoke("add_project", { value: project }), { preserveMode: true });
  }

  async function generateTimesheet() {
    await generateTimesheetExport("all", "full");
  }

  async function generateTimesheetExport(range: TimesheetRange, format: TimesheetFormat = "full") {
    log.info("generateTimesheetExport", { range, format });
    try {
      await invoke("generate_timesheet_export", { range, format });
      await loadState({ preserveMode: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      log.warn("generateTimesheetExport failed", { range, format, message });
      alert(message);
    }
  }

  async function openTimesheetPreview(range: TimesheetRange, format: TimesheetFormat = "full") {
    if (!isTimesheetPreviewWindow) {
      await invoke("open_timesheet_preview_window", { range, format });
      return;
    }
    try {
      const preview = await invoke<TimesheetPreview>("preview_timesheet", { range, format });
      timesheetPreview = preview;
      timesheetPreviewRange = range;
      timesheetPreviewFormat = format;
      timesheetPreviewSheetIndex = 0;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      log.warn("openTimesheetPreview failed", { range, format, message });
      alert(message);
    }
  }

  async function closeTimesheetPreviewWindow() {
    await getCurrentWindow().close().catch(() => {});
  }

  async function toggleTimesheetRounding() {
    timesheetRoundingEnabled = !timesheetRoundingEnabled;
    await persistUiSettings();
  }

  async function resetTimesheet() {
    if (confirm("Reset the timesheet?")) {
      log.warn("resetTimesheet confirmed");
      await refreshFromCommand(invoke("reset_timesheet"), { preserveMode: true });
    }
  }

  async function resetProjects() {
    if (confirm("Reset all saved projects?")) {
      log.warn("resetProjects confirmed");
      await refreshFromCommand(invoke("reset_projects"), { preserveMode: true });
    }
  }

  async function toggleAlwaysOnTop() {
    alwaysOnTop = !alwaysOnTop;
    await getCurrentWindow().setAlwaysOnTop(alwaysOnTop);
    await persistUiSettings();
    log.info("toggleAlwaysOnTop", { alwaysOnTop });
  }

  async function toggleOpenOnStart() {
    openOnStart = !openOnStart;
    await persistUiSettings();
    log.info("toggleOpenOnStart", { openOnStart });
  }

  async function setQuickPanelMode(nextMode: QuickPanelMode) {
    if (quickPanelMode === nextMode) return;
    quickPanelMode = nextMode;
    await persistUiSettings();
    await applyQuickpanelModeLayout(nextMode);
    log.info("setQuickPanelMode", { quickPanelMode: nextMode });
  }

  async function toggleQuickPanelMode() {
    await setQuickPanelMode(quickPanelMode === "compact" ? "normal" : "compact");
  }

  function queueSettingsSave() {
    if (settingsSaveTimer) clearTimeout(settingsSaveTimer);
    settingsSaveTimer = setTimeout(() => {
      persistUiSettings().catch(() => {});
    }, 160);
  }

  async function checkForUpdate() {
    log.info("checkForUpdate");
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateVersion = update.version;
        updateStatus = "available";
        await invoke("set_update_available", { available: true });
        log.info("updateAvailable", { version: update.version });
      } else {
        pendingUpdate = null;
        updateVersion = "";
        updateStatus = "idle";
        await invoke("set_update_available", { available: false });
      }
    } catch (error) {
      log.warn("checkForUpdate failed", error);
      updateStatus = "idle";
      await invoke("set_update_available", { available: false }).catch(() => {});
    }
  }

  async function openUpdatePrompt() {
    if (!pendingUpdate && updateStatus !== "downloading" && updateStatus !== "ready") {
      await checkForUpdate();
    }
    updatePromptOpen = true;
  }

  function closeUpdatePrompt() {
    if (updateStatus !== "downloading" && updateStatus !== "ready") {
      updatePromptOpen = false;
    }
  }

  async function installUpdate() {
    if (!pendingUpdate) return;
    log.info("installUpdate", { version: pendingUpdate.version });
    updatePromptOpen = true;
    updateStatus = "downloading";
    updateProgress = 0;
    let totalBytes = 0;
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") totalBytes = event.data.contentLength ?? 0;
      if (event.event === "Progress" && totalBytes > 0) {
        updateProgress = Math.min(updateProgress + (event.data.chunkLength / totalBytes) * 100, 100);
      }
      if (event.event === "Finished") updateStatus = "ready";
    });
    log.info("updateInstalledRelaunching");
    await relaunch();
  }

  async function submitDialog() {
    await invoke("submit_input", { mode, value: value.trim() });
    dialogOpen = false;
    value = "";
    await loadState();
  }

  function cancelDialog() {
    dialogOpen = false;
    value = "";
  }

  function openAbout() {
    aboutOpen = true;
  }

  function closeAbout() {
    aboutOpen = false;
  }

  function onKeydown(e: KeyboardEvent) {
    if (dialogOpen && e.key === "Enter") submitDialog();
    if (dialogOpen && e.key === "Escape") cancelDialog();
  }

  function setSortMode(next: SortMode) {
    sortMode = next;
    queueSettingsSave();
  }

  function handleDragStart(project: string) {
    if (effectiveSortMode !== "manual") return;
    draggedProject = project;
    dropTargetProject = project;
    dropPosition = "before";
  }

  function handleDragOver(event: MouseEvent, project: string) {
    if (effectiveSortMode !== "manual" || !draggedProject) return;
    const row = event.currentTarget as HTMLElement | null;
    if (row) {
      const rect = row.getBoundingClientRect();
      dropPosition = event.clientY < rect.top + rect.height / 2 ? "before" : "after";
    }
    dropTargetProject = project;
  }

  function finishDrag() {
    if (effectiveSortMode !== "manual" || !draggedProject) return;
    const droppedProject = draggedProject;
    const project = dropTargetProject;
    if (!project || droppedProject === project) {
      draggedProject = null;
      dropTargetProject = null;
      return;
    }

    const order = [...manualOrder];
    const from = order.indexOf(droppedProject);
    const targetIndex = order.indexOf(project);
    let to = targetIndex;
    if (from === -1 || to === -1) return;

    if (dropPosition === "after") {
      to += 1;
    }

    order.splice(from, 1);
    if (from < to) {
      to -= 1;
    }
    order.splice(to, 0, droppedProject);
    manualOrder = order;
    queueSettingsSave();
    draggedProject = null;
    dropTargetProject = null;
    dropPosition = "before";
  }

  onMount(async () => {
    log.info("mounted");
    await loadState();
    if (isTimesheetPreviewWindow) {
      const range = (windowParams.get("range") as TimesheetRange | null) ?? "all";
      const format = (windowParams.get("format") as TimesheetFormat | null) ?? "full";
      await openTimesheetPreview(range, format);
      return;
    }
    const win = getCurrentWindow();
    await restoreQuickPanelBounds();
    await applyQuickpanelModeLayout(quickPanelMode);
    await win.setAlwaysOnTop(alwaysOnTop);
    await checkForUpdate();
    if (openOnStart) await win.show();

    let lastBounds = "";
    const interval = setInterval(async () => {
      try {
        const pos = await win.outerPosition();
        const size = await win.outerSize();
        const next = `${pos.x}:${pos.y}:${size.width}:${size.height}`;
        if (next !== lastBounds) {
          lastBounds = next;
          await invoke("save_quickpanel_bounds", {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
          });
        }
      } catch {
      }
    }, 800);

    const unlistenInput = listen<{ mode: string; title: string; value: string; closeOnSubmit?: boolean }>("show-input", (event) => {
      mode = event.payload.mode;
      title = event.payload.title;
      value = event.payload.value;
      closeInputOnSubmit = event.payload.closeOnSubmit ?? true;
      dialogOpen = true;
      setTimeout(() => inputEl?.focus(), 50);
    });
    const unlistenAbout = listen("show-about", openAbout);
    const unlistenTimesheetPreview = listen<{ range: TimesheetRange; format: TimesheetFormat }>("show-timesheet-preview", (event) => {
      openTimesheetPreview(event.payload.range, event.payload.format);
    });
    const unlistenState = listen("state-changed", () => {
      if (ignoredStateChangedEvents > 0) {
        ignoredStateChangedEvents -= 1;
        return;
      }
      loadState();
    });
    const unlistenUpdatePrompt = listen("show-update-prompt", openUpdatePrompt);
    const unlistenSubmitted = listen("input-submitted", () => {
      dialogOpen = false;
      value = "";
      if (closeInputOnSubmit) {
        getCurrentWindow().hide().catch(() => {});
      }
    });

    return () => {
      clearInterval(interval);
      unlistenInput.then((fn) => fn());
      unlistenAbout.then((fn) => fn());
      unlistenTimesheetPreview.then((fn) => fn());
      unlistenState.then((fn) => fn());
      unlistenUpdatePrompt.then((fn) => fn());
      unlistenSubmitted.then((fn) => fn());
    };
  });

  let allProjects = $derived.by(() => {
    const combined = [...appState.projects, ...appState.adhoc_projects];
    if (effectiveSortMode === "alphabetical") {
      return [...combined].sort((a, b) => a.localeCompare(b));
    }
    if (effectiveSortMode === "recent") {
      return [...combined].sort((a, b) => (recentProjects[b] ?? 0) - (recentProjects[a] ?? 0) || a.localeCompare(b));
    }
    return manualOrder.filter((project) => combined.includes(project));
  });

  let displayedTimesheetSheet = $derived(
    timesheetPreview ? timesheetPreview.sheets[timesheetPreviewSheetIndex] : null
  );

  let displayedTimesheetRows = $derived.by(() => {
    if (!displayedTimesheetSheet) return [];
    const rows = displayedTimesheetSheet.rows.map((row) => {
      if (!timesheetRoundingEnabled || row.is_total) {
        return row;
      }
      const roundedValues = roundHalfPreservingSum(row.values);
      return {
        ...row,
        values: roundedValues,
        total: roundedValues.reduce((sum, value) => sum + value, 0),
      };
    });
    if (!timesheetRoundingEnabled) {
      return rows;
    }
    const totalIndex = rows.findIndex((row) => row.is_total);
    if (totalIndex === -1) {
      return rows;
    }
    const valueCount = rows[totalIndex].values.length;
    const columnTotals = new Array<number>(valueCount).fill(0);
    for (const row of rows) {
      if (row.is_total) continue;
      row.values.forEach((value, index) => {
        columnTotals[index] += value;
      });
    }
    rows[totalIndex] = {
      ...rows[totalIndex],
      values: columnTotals,
      total: columnTotals.reduce((sum, value) => sum + value, 0),
    };
    return rows;
  });
</script>

<svelte:window onkeydown={onKeydown} onmouseup={finishDrag} />

<main
  class:compact-shell={!isTimesheetPreviewWindow && quickPanelMode === "compact"}
  class:timesheet-window={isTimesheetPreviewWindow}
  style:opacity={isTimesheetPreviewWindow ? 1 : quickPanelOpacity}
>
  {#if isTimesheetPreviewWindow}
    {#if timesheetPreview && displayedTimesheetSheet}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <header class="timesheet-window-header" onmousedown={startWindowDrag}>
        <div>
          <h1>{timesheetPreview.title}</h1>
          <p>
            {#if timesheetPreviewFormat === "recent"}
              Yesterday + today overview
            {:else}
              Weekly tabs from the full ProjectLog history
            {/if}
          </p>
        </div>
      </header>

      <section class="timesheet-preview-panel">
        {#if timesheetPreview.sheets.length > 1}
          <div class="sheet-tabs">
            {#each timesheetPreview.sheets as sheet, index}
              <button
                class:sort-active={timesheetPreviewSheetIndex === index}
                onclick={() => (timesheetPreviewSheetIndex = index)}
              >{sheet.name}</button>
            {/each}
          </div>
        {/if}

        <div class="timesheet-table-wrap">
          <table class="timesheet-table">
            <thead>
              <tr>
                <th>Project</th>
                {#each displayedTimesheetSheet.columns as column}
                  <th>{column}</th>
                {/each}
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {#each displayedTimesheetRows as row}
                <tr class:comment-row={row.is_comment} class:total-row={row.is_total}>
                  <td>{row.label}</td>
                  {#each row.values as value}
                    <td>{formatHours(value)}</td>
                  {/each}
                  <td>{formatHours(row.total)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>

      <section class="timesheet-window-footer">
        <div class="toggle-row">
          <span>Round to 0.5h</span>
          <button aria-label="Round to 0.5h" class:toggle-on={timesheetRoundingEnabled} class="toggle" onclick={toggleTimesheetRounding}><span></span></button>
        </div>
        <div class="dialog-buttons about-buttons">
          <button onclick={closeTimesheetPreviewWindow}>Close</button>
          <button class="primary" onclick={() => generateTimesheetExport(timesheetPreviewRange, timesheetPreviewFormat)}>Export to Excel</button>
        </div>
      </section>
    {:else}
      <section class="timesheet-preview-panel timesheet-preview-loading">
        <h1>Loading timesheet…</h1>
        <p>ProjectLog is preparing the preview window.</p>
      </section>
    {/if}
  {:else}
    {#if updateStatus !== "idle"}
      <section class="update">
        <div>
          {#if updateStatus === "available"}Update available: v{updateVersion}{/if}
          {#if updateStatus === "downloading"}Updating... {Math.round(updateProgress)}%{/if}
          {#if updateStatus === "ready"}Update installed. Restarting...{/if}
        </div>
        {#if updateStatus === "available"}
          <button class="primary small" onclick={openUpdatePrompt}>Details</button>
        {/if}
      </section>
    {/if}

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="quickpanel-header">
      <div class="quickpanel-drag" onmousedown={startWindowDrag}>
        <img class="logo" src={appIcon} alt="ProjectLog" />
        <div>
          <h1>ProjectLog QuickPanel</h1>
          <p>{appState.active_project || "No active project"}</p>
          {#if quickPanelMode === "normal" && appState.active_comment}
            <p class="active-comment">{appState.active_comment}</p>
          {/if}
        </div>
      </div>
      <div class="header-actions">
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={() => getCurrentWindow().hide()}
        >Hide</button>
      </div>
    </header>

  <section class="panel project-panel">
    {#if quickPanelMode === "normal"}
      <div class="sort-row">
        <button class:sort-active={sortMode === "manual"} onclick={() => setSortMode("manual")}>Manual</button>
        <button class:sort-active={sortMode === "alphabetical"} onclick={() => setSortMode("alphabetical")}>A-Z</button>
        <button class:sort-active={sortMode === "recent"} onclick={() => setSortMode("recent")}>Recent</button>
      </div>
    {/if}
    <div class="project-list">
      {#if allProjects.length === 0}
        <div class="empty">No projects yet.</div>
      {/if}
      {#each allProjects as project}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class:active={appState.active_project === project}
          class:manual-sort={effectiveSortMode === "manual"}
          class:dragging={draggedProject === project}
          class:drop-target={dropTargetProject === project}
          class:drop-after={dropTargetProject === project && dropPosition === "after"}
          class="project-row"
          onmousemove={(event) => handleDragOver(event, project)}
        >
          {#if effectiveSortMode === "manual"}
            <button
              class="drag-handle"
              aria-label="Reorder project"
              title="Drag to reorder"
              onmousedown={(event) => {
                event.preventDefault();
                event.stopPropagation();
                handleDragStart(project);
              }}
            >≡</button>
          {/if}
          <button class="project-button" onclick={() => selectProject(project)}>
            <span>{project}</span>
            {#if appState.active_project === project}<strong>Active</strong>{/if}
          </button>
          {#if appState.projects.includes(project)}
            <button class="icon-button" title="Remove project" onclick={() => removeProject(project)}>x</button>
          {:else}
            <button class="icon-button icon-add" title="Save project" onclick={() => saveAdhocProject(project)}>+</button>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  {#if quickPanelMode === "normal"}
    <section class="panel compact">
      <input
        id="comment"
        bind:value={commentText}
        disabled={!appState.active_project}
        placeholder="Comment"
        onkeydown={(event) => isEnterSubmit(event) && saveComment()}
      />
      <button onclick={saveComment} disabled={!appState.active_project}>Save</button>
      <button onclick={clearComment} disabled={!appState.active_project || !commentText}>Clear</button>
    </section>

    <section class="panel compact">
      <input
        bind:value={newProjectName}
        placeholder="Add project"
        onkeydown={(event) => isEnterSubmit(event) && addProject()}
      />
      <button onclick={addProject}>Add</button>
    </section>

    <section class="panel compact">
      <input
        bind:value={quickName}
        placeholder="Quick project"
        onkeydown={(event) => isEnterSubmit(event) && trackQuick(false)}
      />
      <button onclick={() => trackQuick(false)}>Track</button>
    </section>

    <section class="actions">
      <button onclick={() => openTimesheetPreview("all")}>Full timesheet</button>
      <button onclick={() => openTimesheetPreview("today", "recent")}>Yesterday + today</button>
      <button onclick={() => invoke("open_log_file")}>Open log file</button>
      <button onclick={resetTimesheet}>Reset timesheet</button>
      <button onclick={resetProjects}>Reset projects</button>
    </section>

  {/if}

  <section class="panel footer" class:compact-footer={quickPanelMode === "compact"}>
    {#if quickPanelMode === "normal"}
      <div class="toggle-row">
        <span>Always on top</span>
        <button aria-label="Always on top" class:toggle-on={alwaysOnTop} class="toggle" onclick={toggleAlwaysOnTop}><span></span></button>
      </div>
      <div class="toggle-row">
        <span>Open QuickPanel on start</span>
        <button aria-label="Open QuickPanel on start" class:toggle-on={openOnStart} class="toggle" onclick={toggleOpenOnStart}><span></span></button>
      </div>
      <div class="opacity-row">
        <span>Opacity</span>
        <input
          type="range"
          min={Math.round(MIN_OPACITY * 100)}
          max="100"
          step="1"
          value={Math.round(quickPanelOpacity * 100)}
          oninput={(event) => {
            const target = event.currentTarget as HTMLInputElement;
            quickPanelOpacity = Number(target.value) / 100;
            queueSettingsSave();
          }}
        />
        <strong>{Math.round(quickPanelOpacity * 100)}%</strong>
      </div>
      <div class="feedback">
        <button onclick={openAbout}>About</button>
      </div>
    {/if}
    <div class="toggle-row">
      <span>Compact mode</span>
      <button aria-label="Compact mode" class:toggle-on={quickPanelMode === "compact"} class="toggle" onclick={toggleQuickPanelMode}><span></span></button>
    </div>
  </section>

  {#if dialogOpen}
    <div class="dialog-backdrop">
      <div class="dialog">
        <h2>{title || "Input"}</h2>
        <input bind:this={inputEl} bind:value type="text" placeholder="Type here..." />
        <div class="dialog-buttons">
          <button onclick={cancelDialog}>Cancel</button>
          <button class="primary" onclick={submitDialog}>OK</button>
        </div>
      </div>
    </div>
  {/if}

  {#if aboutOpen}
    <div class="dialog-backdrop">
      <div class="dialog about-dialog">
        <h2>About ProjectLog</h2>
        <p class="about-copy">ProjectLog helps you track project changes with as little friction as possible.</p>
        <p class="about-meta">Version {appState.app_version}</p>
        <p class="about-meta">Developed by Karsten Garborg.</p>
        <p class="about-links">
          <a href="https://github.com/kargaen/ProjectLog" onclick={(event) => {
            event.preventDefault();
            invoke("open_project_homepage");
          }}>ProjectLog</a>
          <span> | </span>
          <a href="mailto:karga@karga.dk" onclick={(event) => {
            event.preventDefault();
            invoke("open_feedback");
          }}>Send feedback by mail</a>
          <span> | </span>
          <a href="https://karga.dk" onclick={(event) => {
            event.preventDefault();
            invoke("open_portfolio");
          }}>Portfolio</a>
        </p>
        <p class="about-links">
          <a href="/diagnostic-log" onclick={(event) => {
            event.preventDefault();
            invoke("open_diagnostic_log");
          }}>Open diagnostic log</a>
        </p>
        <div class="dialog-buttons about-buttons">
          <button class="primary" onclick={closeAbout}>Close</button>
        </div>
      </div>
    </div>
  {/if}

  {#if updatePromptOpen}
    <div class="dialog-backdrop">
      <div class="dialog about-dialog">
        <h2>ProjectLog update</h2>
        {#if updateStatus === "available"}
          <p class="about-copy">Version {updateVersion} is ready. ProjectLog can download and install it for you.</p>
          <p class="about-meta">Release notes open on the GitHub releases page.</p>
          <div class="dialog-buttons about-buttons">
            <button onclick={() => invoke("open_release_notes")}>Release notes</button>
            <button onclick={closeUpdatePrompt}>Later</button>
            <button class="primary" onclick={installUpdate}>Update now</button>
          </div>
        {:else if updateStatus === "downloading"}
          <p class="about-copy">Downloading and installing the update.</p>
          <p class="about-meta">{Math.round(updateProgress)}%</p>
        {:else if updateStatus === "ready"}
          <p class="about-copy">Update installed. ProjectLog is restarting.</p>
        {:else}
          <p class="about-copy">No update is ready right now.</p>
          <div class="dialog-buttons about-buttons">
            <button class="primary" onclick={() => (updatePromptOpen = false)}>Close</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
  {/if}

</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: transparent;
    color: #20241f;
  }

  main {
    position: relative;
    box-sizing: border-box;
    height: 100vh;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: #f4f5f1;
    border: 0;
    border-radius: 10px;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.16);
    overflow: hidden;
  }

  header,
  .panel,
  .actions,
  .update {
    border: 1px solid #d8dccc;
    background: #ffffff;
    border-radius: 5px;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 7px 8px;
  }

  .quickpanel-header {
    user-select: none;
  }

  .header-actions {
    display: flex;
    gap: 4px;
  }

  .quickpanel-drag {
    min-width: 0;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: move;
  }

  .logo {
    width: 22px;
    height: 22px;
    flex: none;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    font-size: 12px;
    font-weight: 650;
  }

  h2 {
    font-size: 13px;
  }

  p {
    margin-top: 1px;
    color: #697064;
    font-size: 10px;
  }

  .panel {
    padding: 5px;
  }

  .project-panel {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    min-height: 0;
    overflow: hidden;
  }

  .compact-shell .project-panel {
    padding-bottom: 2px;
  }

  .sort-row {
    display: flex;
    gap: 4px;
    margin-bottom: 5px;
  }

  .sort-row button {
    padding: 4px 6px;
    background: #fff;
    color: #20241f;
  }

  .sort-active {
    border-color: #267a62;
    background: #e9f6f1;
    color: #267a62;
  }

  .project-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-height: 0;
    padding-top: 3px;
    padding-bottom: 3px;
    overflow-y: auto;
  }

  .compact-shell .project-list {
    padding-top: 0;
    padding-bottom: 0;
  }

  .project-row {
    display: grid;
    grid-template-columns: 1fr 26px;
    gap: 4px;
    align-items: center;
    position: relative;
  }

  .project-row.dragging {
    opacity: 0.55;
  }

  .project-row.manual-sort {
    grid-template-columns: 18px 1fr 26px;
  }

  .project-row:not(:has(.icon-button)) {
    grid-template-columns: 1fr;
  }

  .project-row.manual-sort:not(:has(.icon-button)) {
    grid-template-columns: 18px 1fr;
  }

  .project-row.drop-target::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: -3px;
    height: 2px;
    background: #267a62;
    border-radius: 999px;
  }

  .project-row.drop-target.drop-after::before {
    top: auto;
    bottom: -3px;
  }

  .drag-handle {
    width: 18px;
    height: 26px;
    padding: 0;
    border: 0;
    background: transparent;
    color: #8a9187;
    cursor: grab;
    font-size: 12px;
    line-height: 1;
  }

  .drag-handle:hover:not(:disabled) {
    background: transparent;
    color: #267a62;
  }

  .project-button {
    min-height: 26px;
    justify-content: space-between;
    text-align: left;
    font-size: 11px;
  }

  .project-row.active .project-button {
    border-color: #267a62;
    background: #e9f6f1;
  }

  strong {
    font-size: 10px;
    color: #267a62;
  }

  .empty {
    padding: 8px 4px;
    color: #697064;
    font-size: 11px;
  }

  .compact {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .compact button {
    flex: 0 0 auto;
  }

  input {
    min-width: 0;
    flex: 1;
    padding: 5px 7px;
    border: 1px solid #c9cebf;
    border-radius: 4px;
    font-size: 11px;
    outline: none;
    background: #fff;
  }

  input:focus {
    border-color: #267a62;
    box-shadow: 0 0 0 2px rgba(38, 122, 98, 0.15);
  }

  input:disabled {
    background: #f2f3ef;
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid #c9cebf;
    border-radius: 4px;
    background: #fff;
    color: #20241f;
    padding: 5px 7px;
    font-size: 11px;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: #f2f5ee;
  }

  button:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .primary {
    border-color: #267a62;
    background: #267a62;
    color: #fff;
  }

  .small {
    padding: 4px 7px;
  }

  .ghost {
    color: #697064;
  }

  .icon-button {
    width: 26px;
    height: 26px;
    padding: 0;
    color: #7a3d35;
    font-size: 11px;
  }

  .icon-add {
    color: #267a62;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 5px;
  }

  .actions button {
    flex: 0 0 auto;
  }

  .footer {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .toggle-row,
  .feedback,
  .update {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .toggle-row span,
  .update div,
  .opacity-row span {
    flex: 1;
    font-size: 11px;
    color: #495047;
  }

  .opacity-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .opacity-row input[type="range"] {
    flex: 1;
    padding: 0;
    border: 0;
    box-shadow: none;
    background: transparent;
  }

  .opacity-row strong {
    min-width: 34px;
    text-align: right;
    color: #495047;
    font-size: 10px;
  }

  .toggle {
    width: 30px;
    height: 18px;
    border-radius: 10px;
    padding: 0;
    justify-content: flex-start;
    background: #d8dccc;
  }

  .toggle:hover:not(:disabled) {
    background: #d8dccc;
  }

  .toggle span {
    flex: none;
    width: 12px;
    height: 12px;
    margin-left: 2px;
    border-radius: 50%;
    background: #fff;
    transition: margin-left 120ms ease;
  }

  .toggle.toggle-on {
    background: #267a62;
  }

  .toggle.toggle-on:hover:not(:disabled) {
    background: #267a62;
  }

  .toggle.toggle-on span {
    margin-left: 14px;
  }

  .feedback button {
    width: 100%;
  }

  .about-copy,
  .about-meta {
    color: #495047;
    font-size: 11px;
    line-height: 1.45;
  }

  .about-buttons {
    justify-content: flex-end;
  }

  .about-links {
    margin-top: 2px;
    font-size: 11px;
  }

  .about-links a {
    color: #267a62;
    text-decoration: none;
  }

  .about-links a:hover {
    text-decoration: underline;
  }

  .about-dialog {
    width: min(360px, 100%);
  }

  .timesheet-dialog {
    width: min(980px, 100%);
    max-height: min(720px, calc(100vh - 32px));
  }

  .timesheet-window {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    background: linear-gradient(180deg, #f5f7f1 0%, #eef1e8 100%);
    box-sizing: border-box;
  }

  .timesheet-window-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 4px 2px 0;
    cursor: default;
  }

  .timesheet-window-header h1 {
    margin: 0;
    color: #21352c;
    font-size: 20px;
  }

  .timesheet-window-header p {
    margin: 4px 0 0;
    color: #5f675d;
    font-size: 12px;
  }

  .timesheet-preview-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid #cfd5c6;
    border-radius: 8px;
    box-shadow: 0 14px 34px rgba(45, 55, 39, 0.1);
  }

  .timesheet-preview-loading {
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .timesheet-preview-loading h1 {
    margin: 0;
    color: #21352c;
    font-size: 18px;
  }

  .timesheet-preview-loading p {
    margin: 4px 0 0;
    color: #5f675d;
    font-size: 12px;
  }

  .sheet-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .timesheet-table-wrap {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border: 1px solid #d8dccc;
    border-radius: 5px;
  }

  .timesheet-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
    background: #fff;
  }

  .timesheet-table th,
  .timesheet-table td {
    padding: 6px 8px;
    border-bottom: 1px solid #ecefe6;
    text-align: right;
    white-space: nowrap;
  }

  .timesheet-table th:first-child,
  .timesheet-table td:first-child {
    text-align: left;
    min-width: 220px;
  }

  .timesheet-table thead th {
    position: sticky;
    top: 0;
    background: #f4f5f1;
    z-index: 1;
  }

  .comment-row td:first-child {
    color: #697064;
  }

  .total-row td {
    font-weight: 650;
    background: #f8faf6;
  }

  .timesheet-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .timesheet-window-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-top: 2px;
  }

  .update {
    padding: 6px 8px;
    background: #e9f6f1;
    border-color: #9bcab9;
  }

  .active-comment {
    color: #267a62;
  }

  .dialog-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    background: rgba(32, 36, 31, 0.25);
  }

  .dialog {
    width: min(320px, 100%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: #fff;
    border: 1px solid #c9cebf;
    border-radius: 6px;
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.2);
  }

  .dialog-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

</style>
