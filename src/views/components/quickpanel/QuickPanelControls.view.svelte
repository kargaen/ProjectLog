<script lang="ts">
  import { tick } from "svelte";
  import type {
    TimesheetFormat,
    TimesheetRange,
  } from "../../../models/types";

  let {
    activeProject,
    commentText,
    newProjectName,
    quickName,
    alwaysOnTop,
    openOnStart,
    minOpacity,
    quickPanelOpacity,
    onCommentTextChange,
    onSaveComment,
    onClearComment,
    onNewProjectNameChange,
    onAddProject,
    onQuickNameChange,
    onTrackQuick,
    onOpenTimesheetPreview,
    onOpenLogFile,
    onResetTimesheet,
    onResetProjects,
    onToggleAlwaysOnTop,
    onToggleOpenOnStart,
    onQuickPanelOpacityChange,
    onOpenAbout,
  }: {
    activeProject: string;
    commentText: string;
    newProjectName: string;
    quickName: string;
    alwaysOnTop: boolean;
    openOnStart: boolean;
    minOpacity: number;
    quickPanelOpacity: number;
    onCommentTextChange: (value: string) => void;
    onSaveComment: () => void | Promise<void>;
    onClearComment: () => void | Promise<void>;
    onNewProjectNameChange: (value: string) => void;
    onAddProject: () => void | Promise<void>;
    onQuickNameChange: (value: string) => void;
    onTrackQuick: () => void | Promise<void>;
    onOpenTimesheetPreview: (
      range: TimesheetRange,
      format?: TimesheetFormat
    ) => void | Promise<void>;
    onOpenLogFile: () => void | Promise<void>;
    onResetTimesheet: () => void | Promise<void>;
    onResetProjects: () => void | Promise<void>;
    onToggleAlwaysOnTop: () => void | Promise<void>;
    onToggleOpenOnStart: () => void | Promise<void>;
    onQuickPanelOpacityChange: (value: number) => void;
    onOpenAbout: () => void;
  } = $props();

  function isEnterSubmit(event: KeyboardEvent) {
    return event.key === "Enter" && !event.shiftKey;
  }

  let overflowOpen = $state(false);
  let overflowTrigger = $state<HTMLButtonElement>();
  let firstOverflowItem = $state<HTMLButtonElement>();

  async function openOverflow() {
    overflowOpen = true;
    await tick();
    firstOverflowItem?.focus();
  }

  function closeOverflow({ restoreFocus = false } = {}) {
    overflowOpen = false;
    if (restoreFocus) {
      overflowTrigger?.focus();
    }
  }

  function handleOverflowTriggerKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      void openOverflow();
    }
  }

  function handleOverflowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeOverflow({ restoreFocus: true });
    }
  }
</script>

<div class="normal-controls-scroll">
  <section class="panel compact">
    <input
      id="comment"
      value={commentText}
      disabled={!activeProject}
      placeholder="Comment"
      oninput={(event) =>
        onCommentTextChange((event.currentTarget as HTMLInputElement).value)}
      onkeydown={(event) => isEnterSubmit(event) && onSaveComment()}
    />
    <button onclick={onSaveComment} disabled={!activeProject}>Save</button>
    <button onclick={onClearComment} disabled={!activeProject || !commentText}>
      Clear
    </button>
  </section>

  <section class="panel compact">
    <input
      value={newProjectName}
      placeholder="Add project"
      oninput={(event) =>
        onNewProjectNameChange((event.currentTarget as HTMLInputElement).value)}
      onkeydown={(event) => isEnterSubmit(event) && onAddProject()}
    />
    <button onclick={onAddProject}>Add</button>
  </section>

  <section class="panel compact">
    <input
      value={quickName}
      placeholder="Quick project"
      oninput={(event) =>
        onQuickNameChange((event.currentTarget as HTMLInputElement).value)}
      onkeydown={(event) => isEnterSubmit(event) && onTrackQuick()}
    />
    <button onclick={onTrackQuick}>Track</button>
  </section>

  <section class="actions">
    <button class="primary" onclick={() => onOpenTimesheetPreview("today", "recent")}>
      Yesterday + today
    </button>
    <button class="primary" onclick={() => onOpenTimesheetPreview("all")}>
      Full timesheet
    </button>
    <button onclick={onOpenLogFile}>Open log file</button>
    <div class="action-overflow">
      <button
        bind:this={overflowTrigger}
        aria-haspopup="menu"
        aria-expanded={overflowOpen}
        onclick={() => overflowOpen ? closeOverflow() : void openOverflow()}
        onkeydown={handleOverflowTriggerKeydown}
      >⋮ More</button>
      {#if overflowOpen}
        <div class="action-overflow-menu" role="menu" tabindex="-1" onkeydown={handleOverflowKeydown}>
          <button
            bind:this={firstOverflowItem}
            role="menuitem"
            onclick={() => {
              closeOverflow();
              onOpenAbout();
            }}
          >About</button>
          <div class="action-overflow-separator" role="separator"></div>
          <button
            class="destructive"
            role="menuitem"
            onclick={() => {
              closeOverflow();
              void onResetTimesheet();
            }}
          >Reset timesheet</button>
          <button
            class="destructive"
            role="menuitem"
            onclick={() => {
              closeOverflow();
              void onResetProjects();
            }}
          >Reset projects</button>
        </div>
      {/if}
    </div>
  </section>

  <section class="panel footer">
    <div class="toggle-row">
      <span>Always on top</span>
      <button
        aria-label="Always on top"
        class:toggle-on={alwaysOnTop}
        class="toggle"
        onclick={onToggleAlwaysOnTop}
      >
        <span></span>
      </button>
    </div>
    <div class="toggle-row">
      <span>Open QuickPanel on start</span>
      <button
        aria-label="Open QuickPanel on start"
        class:toggle-on={openOnStart}
        class="toggle"
        onclick={onToggleOpenOnStart}
      >
        <span></span>
      </button>
    </div>
    <div class="opacity-row">
      <span>Opacity</span>
      <input
        type="range"
        min={Math.round(minOpacity * 100)}
        max="100"
        step="1"
        value={Math.round(quickPanelOpacity * 100)}
        oninput={(event) =>
          onQuickPanelOpacityChange(
            Number((event.currentTarget as HTMLInputElement).value) / 100
          )}
      />
      <strong>{Math.round(quickPanelOpacity * 100)}%</strong>
    </div>
  </section>
</div>
