<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import {
    availableMonitors,
    getCurrentWindow,
    PhysicalPosition,
    PhysicalSize,
    primaryMonitor,
  } from "@tauri-apps/api/window";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import TimesheetPreviewWindow from "./lib/components/TimesheetPreviewWindow.svelte";
  import { createLogger } from "./lib/logger";
  import type {
    ProjectState,
    QuickPanelMode,
    SortMode,
    TimesheetFormat,
    TimesheetRange,
  } from "./lib/types";
  import appIcon from "../icon.svg";

  const log = createLogger("quickpanel");
  const MIN_WINDOW_WIDTH = 220;
  const MIN_WINDOW_HEIGHT = 90;
  const AUTO_COMPACT_HEIGHT = 430;
  const MIN_OPACITY = 0.35;

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

  let dialogOpen = $state(false);
  let aboutOpen = $state(false);
  let mode = $state("");
  let title = $state("");
  let value = $state("");
  let inputEl: HTMLInputElement | undefined = $state();
  let settingsSaveTimer: ReturnType<typeof setTimeout> | undefined = $state();
  let closeInputOnSubmit = $state(true);
  let ignoredStateChangedEvents = $state(0);
  let timesheetRoundingEnabled = $state(false);
  let currentWindowHeight = $state(Infinity);

  const windowParams = new URLSearchParams(window.location.search);
  const tauriWindowLabel =
    (window as Window & {
      __TAURI_INTERNALS__?: {
        metadata?: {
          currentWindow?: { label?: string };
        };
      };
    }).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
  const currentWindow = getCurrentWindow();
  const currentWindowLabel = tauriWindowLabel ?? windowParams.get("window") ?? "main";
  const isDedicatedTimesheetPreviewWindow =
    windowParams.get("window") === "timesheet-preview" || currentWindowLabel === "timesheet-preview";
  const isCompactLayout = $derived(
    quickPanelMode === "compact" || currentWindowHeight < AUTO_COMPACT_HEIGHT
  );
  let effectiveSortMode = $derived<SortMode>(isCompactLayout ? "manual" : sortMode);

  async function getMinimumWindowSizePhysical() {
    const scaleFactor = await currentWindow.scaleFactor().catch(() => 1);

    return {
      minWidth: Math.ceil(MIN_WINDOW_WIDTH * scaleFactor),
      minHeight: Math.ceil(MIN_WINDOW_HEIGHT * scaleFactor),
    };
  }

  async function updateWindowHeight(nextHeight?: number) {
    const scaleFactor = await currentWindow.scaleFactor().catch(() => 1);
    if (nextHeight !== undefined) {
      currentWindowHeight = nextHeight / scaleFactor;
      return;
    }

    const size = await currentWindow.outerSize().catch(() => null);
    if (!size) return;
    currentWindowHeight = size.height / scaleFactor;
  }

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

  async function startWindowDrag() {
    await currentWindow.startDragging().catch(() => {});
  }

  async function restoreQuickPanelBounds() {
    const savedWidth = appState.settings.quickpanel_width;
    const savedHeight = appState.settings.quickpanel_height;
    const savedX = appState.settings.quickpanel_x;
    const savedY = appState.settings.quickpanel_y;
    const { minWidth, minHeight } = await getMinimumWindowSizePhysical();

    if (!savedWidth || !savedHeight) {
      return;
    }

    const monitors = await availableMonitors().catch(() => []);
    const fallbackMonitor = (await primaryMonitor().catch(() => null)) ?? monitors[0] ?? null;

    if (!fallbackMonitor) {
      await currentWindow
        .setSize(new PhysicalSize(Math.max(savedWidth, minWidth), Math.max(savedHeight, minHeight)))
        .catch(() => {});
      if (savedX !== null && savedY !== null) {
        await currentWindow.setPosition(new PhysicalPosition(savedX, savedY)).catch(() => {});
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
    const width = clamp(savedWidth, minWidth, area.size.width);
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

    await currentWindow.setSize(new PhysicalSize(width, height)).catch(() => {});
    await currentWindow.setPosition(new PhysicalPosition(x, y)).catch(() => {});
  }

  function applyAppState(nextState: ProjectState, options?: { preserveMode?: boolean }) {
    const preserveMode = options?.preserveMode ?? false;
    const currentQuickPanelMode = quickPanelMode;
    const currentSortMode = sortMode;

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
    appState.settings.project_sort_mode = preserveMode
      ? currentSortMode
      : nextState.settings.project_sort_mode;
    appState.settings.quickpanel_mode = nextState.settings.quickpanel_mode;
    appState.settings.project_manual_order = nextState.settings.project_manual_order;
    appState.settings.project_recent_usage = nextState.settings.project_recent_usage;
    appState.settings.timesheet_rounding_enabled = nextState.settings.timesheet_rounding_enabled;

    commentText = nextState.active_comment;
    alwaysOnTop = nextState.settings.always_on_top;
    openOnStart = nextState.settings.open_on_start;
    quickPanelOpacity = nextState.settings.quickpanel_opacity;
    sortMode = preserveMode ? currentSortMode : (nextState.settings.project_sort_mode ?? "manual");
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

  async function applyQuickpanelModeLayout() {
    await currentWindow
      .setSizeConstraints({
        minWidth: MIN_WINDOW_WIDTH,
        minHeight: MIN_WINDOW_HEIGHT,
      })
      .catch(() => {});
    await updateWindowHeight().catch(() => {});
  }

  async function selectProject(project: string) {
    log.info("selectProject", { project });
    await refreshFromCommand(invoke("select_project", { project }), { preserveMode: true });
  }

  async function addProject() {
    const nextValue = newProjectName.trim();
    if (!nextValue) return;
    log.info("addProject", { length: nextValue.length });
    await refreshFromCommand(invoke("add_project", { value: nextValue }), { preserveMode: true });
    newProjectName = "";
  }

  async function trackQuick(addToo: boolean) {
    const nextValue = quickName.trim();
    if (!nextValue) return;
    log.info("trackQuick", { addToo, length: nextValue.length });
    if (addToo) {
      await refreshFromCommand(invoke("add_project", { value: nextValue }), { preserveMode: true });
    }
    await refreshFromCommand(invoke("quick_project", { value: nextValue }), { preserveMode: true });
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
    await invoke("open_timesheet_preview_window", { range, format });
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
    await currentWindow.setAlwaysOnTop(alwaysOnTop);
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
    await applyQuickpanelModeLayout();
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

  function onKeydown(event: KeyboardEvent) {
    if (dialogOpen && event.key === "Enter") submitDialog();
    if (dialogOpen && event.key === "Escape") cancelDialog();
  }

  function setSortMode(nextMode: SortMode) {
    sortMode = nextMode;
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
    if (!draggedProject) return;
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

  onMount(() => {
    if (isDedicatedTimesheetPreviewWindow) {
      return;
    }

    let disposed = false;
    let cleanup = () => {};

    void (async () => {
      log.info("mounted");
      await loadState();
      await restoreQuickPanelBounds();
      await applyQuickpanelModeLayout();
      await currentWindow.setAlwaysOnTop(alwaysOnTop);
      await checkForUpdate();
      if (openOnStart) await currentWindow.show();

      let lastBounds = "";
      const unlistenResized = currentWindow.onResized(async ({ payload: size }) => {
        await updateWindowHeight(size.height).catch(() => {});
      });

      const interval = setInterval(async () => {
        try {
          const pos = await currentWindow.outerPosition();
          const size = await currentWindow.outerSize();
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

      const unlistenInput = listen<{ mode: string; title: string; value: string; closeOnSubmit?: boolean }>(
        "show-input",
        (event) => {
          mode = event.payload.mode;
          title = event.payload.title;
          value = event.payload.value;
          closeInputOnSubmit = event.payload.closeOnSubmit ?? true;
          dialogOpen = true;
          setTimeout(() => inputEl?.focus(), 50);
        }
      );
      const unlistenAbout = listen("show-about", openAbout);
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
          currentWindow.hide().catch(() => {});
        }
      });

      cleanup = () => {
        clearInterval(interval);
        unlistenResized.then((fn) => fn());
        unlistenInput.then((fn) => fn());
        unlistenAbout.then((fn) => fn());
        unlistenState.then((fn) => fn());
        unlistenUpdatePrompt.then((fn) => fn());
        unlistenSubmitted.then((fn) => fn());
      };

      if (disposed) {
        cleanup();
      }
    })();

    return () => {
      disposed = true;
      cleanup();
    };
  });

  let allProjects = $derived.by(() => {
    const combined = [...appState.projects, ...appState.adhoc_projects];
    if (effectiveSortMode === "alphabetical") {
      return [...combined].sort((a, b) => a.localeCompare(b));
    }
    if (effectiveSortMode === "recent") {
      return [...combined].sort(
        (a, b) => (recentProjects[b] ?? 0) - (recentProjects[a] ?? 0) || a.localeCompare(b)
      );
    }
    return manualOrder.filter((project) => combined.includes(project));
  });
</script>

<svelte:window onkeydown={onKeydown} onmouseup={finishDrag} />

{#if isDedicatedTimesheetPreviewWindow}
  <TimesheetPreviewWindow />
{:else}
  <main class:compact-shell={isCompactLayout} style:opacity={quickPanelOpacity}>
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
          {#if !isCompactLayout && appState.active_comment}
            <p class="active-comment">{appState.active_comment}</p>
          {/if}
        </div>
      </div>
      <div class="header-actions">
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={() => currentWindow.hide()}
        >Hide</button>
      </div>
    </header>

    <section class="panel project-panel">
      {#if !isCompactLayout}
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
              >|||</button>
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

    {#if !isCompactLayout}
      <div class="normal-controls-scroll">
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

        <section class="panel footer">
          <div class="toggle-row">
            <span>Always on top</span>
            <button
              aria-label="Always on top"
              class:toggle-on={alwaysOnTop}
              class="toggle"
              onclick={toggleAlwaysOnTop}
            ><span></span></button>
          </div>
          <div class="toggle-row">
            <span>Open QuickPanel on start</span>
            <button
              aria-label="Open QuickPanel on start"
              class:toggle-on={openOnStart}
              class="toggle"
              onclick={toggleOpenOnStart}
            ><span></span></button>
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
        </section>
      </div>
    {/if}

    <section class="panel footer" class:compact-footer={isCompactLayout}>
      <div class="toggle-row">
        <span>Compact mode</span>
        <button
          aria-label="Compact mode"
          class:toggle-on={isCompactLayout}
          class="toggle"
          onclick={toggleQuickPanelMode}
        ><span></span></button>
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
  </main>
{/if}

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
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  h2 {
    font-size: 13px;
  }

  p {
    margin-top: 1px;
    color: #697064;
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
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

  .normal-controls-scroll {
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    padding-right: 1px;
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
    min-width: 0;
    min-height: 26px;
    justify-content: space-between;
    text-align: left;
    font-size: 11px;
  }

  .project-button span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-row.active .project-button {
    border-color: #267a62;
    background: #e9f6f1;
  }

  strong {
    flex: none;
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
