<script lang="ts">
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
    <button onclick={() => onOpenTimesheetPreview("all")}>
      Full timesheet
    </button>
    <button onclick={() => onOpenTimesheetPreview("today", "recent")}>
      Yesterday + today
    </button>
    <button onclick={onOpenLogFile}>Open log file</button>
    <button onclick={onResetTimesheet}>Reset timesheet</button>
    <button onclick={onResetProjects}>Reset projects</button>
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
    <div class="feedback">
      <button onclick={onOpenAbout}>About</button>
    </div>
  </section>
</div>
