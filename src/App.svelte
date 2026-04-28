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

  const log = createLogger("quickpanel");
  const MIN_QUICKPANEL_WIDTH = 300;
  const MIN_NORMAL_HEIGHT = 430;
  const MIN_COMPACT_HEIGHT = 220;
  const MIN_OPACITY = 0.35;

  type SortMode = "manual" | "alphabetical" | "recent";
  type QuickPanelMode = "normal" | "compact";
  type TimesheetRange = "today" | "week" | "all";
  type TimesheetFormat = "full" | "lite";

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
    };
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

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  function rememberProjectUse(project: string) {
    if (!project) return;
    recentProjects = {
      ...recentProjects,
      [project]: Date.now(),
    };
    queueSettingsSave();
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
    await getCurrentWindow().startDragging().catch(() => {});
  }

  async function startResizeDrag() {
    await getCurrentWindow().startResizeDragging("SouthEast").catch(() => {});
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

  async function loadState() {
    log.debug("loadState");
    appState = await invoke<ProjectState>("get_state");
    commentText = appState.active_comment;
    alwaysOnTop = appState.settings.always_on_top;
    openOnStart = appState.settings.open_on_start;
    quickPanelOpacity = appState.settings.quickpanel_opacity;
    sortMode = appState.settings.project_sort_mode ?? "manual";
    quickPanelMode = appState.settings.quickpanel_mode ?? "normal";
    manualOrder = appState.settings.project_manual_order ?? [];
    recentProjects = appState.settings.project_recent_usage ?? {};
    syncManualOrder([...appState.projects, ...appState.adhoc_projects]);
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
    await invoke("select_project", { project });
    rememberProjectUse(project);
    await loadState();
  }

  async function addProject() {
    const value = newProjectName.trim();
    if (!value) return;
    log.info("addProject", { length: value.length });
    await invoke("add_project", { value });
    newProjectName = "";
    await loadState();
  }

  async function trackQuick(addToo: boolean) {
    const value = quickName.trim();
    if (!value) return;
    log.info("trackQuick", { addToo, length: value.length });
    if (addToo) await invoke("add_project", { value });
    await invoke("quick_project", { value });
    rememberProjectUse(value);
    quickName = "";
    await loadState();
  }

  async function saveComment() {
    log.info("saveComment", { length: commentText.trim().length });
    await invoke("set_comment", { value: commentText.trim() });
    await loadState();
  }

  async function clearComment() {
    commentText = "";
    log.info("clearComment");
    await invoke("set_comment", { value: "" });
    await loadState();
  }

  async function removeProject(project: string) {
    log.warn("removeProject", { project });
    await invoke("remove_project", { project });
    await loadState();
  }

  async function saveAdhocProject(project: string) {
    log.info("saveAdhocProject", { project });
    await invoke("add_project", { value: project });
    await loadState();
  }

  async function generateTimesheet() {
    await generateTimesheetExport("all", "full");
  }

  async function generateTimesheetExport(range: TimesheetRange, format: TimesheetFormat = "full") {
    log.info("generateTimesheetExport", { range, format });
    await invoke("generate_timesheet_export", { range, format });
    await loadState();
  }

  async function resetTimesheet() {
    if (confirm("Reset the timesheet?")) {
      log.warn("resetTimesheet confirmed");
      await invoke("reset_timesheet");
      await loadState();
    }
  }

  async function resetProjects() {
    if (confirm("Reset all saved projects?")) {
      log.warn("resetProjects confirmed");
      await invoke("reset_projects");
      await loadState();
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
    if (sortMode !== "manual") return;
    draggedProject = project;
    dropTargetProject = project;
    dropPosition = "before";
  }

  function handleDragOver(event: MouseEvent, project: string) {
    if (sortMode !== "manual" || !draggedProject) return;
    const row = event.currentTarget as HTMLElement | null;
    if (row) {
      const rect = row.getBoundingClientRect();
      dropPosition = event.clientY < rect.top + rect.height / 2 ? "before" : "after";
    }
    dropTargetProject = project;
  }

  function finishDrag() {
    if (sortMode !== "manual" || !draggedProject) return;
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
    const unlistenState = listen("state-changed", loadState);
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
      unlistenState.then((fn) => fn());
      unlistenUpdatePrompt.then((fn) => fn());
      unlistenSubmitted.then((fn) => fn());
    };
  });

  let allProjects = $derived.by(() => {
    const combined = [...appState.projects, ...appState.adhoc_projects];
    if (sortMode === "alphabetical") {
      return [...combined].sort((a, b) => a.localeCompare(b));
    }
    if (sortMode === "recent") {
      return [...combined].sort((a, b) => (recentProjects[b] ?? 0) - (recentProjects[a] ?? 0) || a.localeCompare(b));
    }
    return manualOrder.filter((project) => combined.includes(project));
  });
</script>

<svelte:window onkeydown={onKeydown} onmouseup={finishDrag} />

<main class:compact-shell={quickPanelMode === "compact"} style:opacity={quickPanelOpacity}>
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

  {#if quickPanelMode === "normal"}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="quickpanel-header">
      <div class="quickpanel-drag" onmousedown={startWindowDrag}>
        <img class="logo" src="/icon.svg" alt="ProjectLog" />
        <div>
          <h1>ProjectLog QuickPanel</h1>
          <p>{appState.active_project || "No active project"}</p>
          {#if appState.active_comment}
            <p class="active-comment">{appState.active_comment}</p>
          {/if}
        </div>
      </div>
      <div class="header-actions">
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={toggleQuickPanelMode}
        >Compact</button>
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={() => getCurrentWindow().hide()}
        >Hide</button>
      </div>
    </header>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="quickpanel-header compact-header">
      <div class="quickpanel-drag" onmousedown={startWindowDrag}>
        <img class="logo" src="/icon.svg" alt="ProjectLog" />
        <div>
          <h1>ProjectLog Compact</h1>
          <p>{appState.active_project || "Pick a project"}</p>
        </div>
      </div>
      <div class="header-actions">
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={toggleQuickPanelMode}
        >Normal</button>
        <button
          class="ghost"
          onmousedown={(event) => event.stopPropagation()}
          onclick={() => getCurrentWindow().hide()}
        >Hide</button>
      </div>
    </header>
  {/if}

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
          class:manual-sort={sortMode === "manual"}
          class:dragging={draggedProject === project}
          class:drop-target={dropTargetProject === project}
          class:drop-after={dropTargetProject === project && dropPosition === "after"}
          class="project-row"
          onmousemove={(event) => handleDragOver(event, project)}
        >
          {#if sortMode === "manual"}
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
      <button onclick={() => generateTimesheetExport("today")}>Today</button>
      <button onclick={() => generateTimesheetExport("week")}>Weekly</button>
      <button onclick={() => generateTimesheetExport("all")}>All data</button>
      <button onclick={() => generateTimesheetExport("today", "lite")}>Generate lite</button>
      <button onclick={() => invoke("open_log_file")}>Open log file</button>
      <button onclick={resetTimesheet}>Reset timesheet</button>
      <button onclick={resetProjects}>Reset projects</button>
    </section>

    <section class="panel footer">
      <div class="toggle-row">
        <span>Always on top</span>
        <button aria-label="Always on top" class:toggle-on={alwaysOnTop} class="toggle" onclick={toggleAlwaysOnTop}><span></span></button>
      </div>
      <div class="toggle-row">
        <span>Open QuickPanel on start</span>
        <button aria-label="Open QuickPanel on start" class:toggle-on={openOnStart} class="toggle" onclick={toggleOpenOnStart}><span></span></button>
      </div>
      <div class="toggle-row">
        <span>Compact mode</span>
        <button aria-label="Compact mode" class:toggle-on={quickPanelMode === "compact"} class="toggle" onclick={toggleQuickPanelMode}><span></span></button>
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
  {/if}

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

  <button
    class="resize-handle"
    aria-label="Resize QuickPanel"
    title="Resize"
    onmousedown={(event) => {
      event.preventDefault();
      startResizeDrag();
    }}
  ></button>
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

  .compact-header {
    padding-block: 6px;
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

  .resize-handle {
    position: absolute;
    right: 6px;
    bottom: 6px;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    background:
      linear-gradient(135deg, transparent 0 45%, rgba(105, 112, 100, 0.8) 45% 55%, transparent 55% 100%),
      linear-gradient(135deg, transparent 0 65%, rgba(105, 112, 100, 0.8) 65% 75%, transparent 75% 100%),
      linear-gradient(135deg, transparent 0 85%, rgba(105, 112, 100, 0.8) 85% 95%, transparent 95% 100%);
    cursor: nwse-resize;
    opacity: 0.85;
  }

  .resize-handle:hover {
    background:
      linear-gradient(135deg, transparent 0 45%, rgba(38, 122, 98, 0.9) 45% 55%, transparent 55% 100%),
      linear-gradient(135deg, transparent 0 65%, rgba(38, 122, 98, 0.9) 65% 75%, transparent 75% 100%),
      linear-gradient(135deg, transparent 0 85%, rgba(38, 122, 98, 0.9) 85% 95%, transparent 95% 100%);
  }
</style>
