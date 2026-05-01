<script lang="ts">
  import type { SortMode } from "../../../models/types";

  let {
    isCompactLayout,
    sortMode,
    allProjects,
    activeProject,
    permanentProjects,
    effectiveSortMode,
    draggedProject,
    dropTargetProject,
    dropPosition,
    onSetSortMode,
    onHandleDragOver,
    onHandleDragStart,
    onSelectProject,
    onRemoveProject,
    onSaveAdhocProject,
  }: {
    isCompactLayout: boolean;
    sortMode: SortMode;
    allProjects: string[];
    activeProject: string;
    permanentProjects: string[];
    effectiveSortMode: SortMode;
    draggedProject: string | null;
    dropTargetProject: string | null;
    dropPosition: "before" | "after";
    onSetSortMode: (mode: SortMode) => void;
    onHandleDragOver: (event: MouseEvent, project: string) => void;
    onHandleDragStart: (project: string) => void;
    onSelectProject: (project: string) => void | Promise<void>;
    onRemoveProject: (project: string) => void | Promise<void>;
    onSaveAdhocProject: (project: string) => void | Promise<void>;
  } = $props();
</script>

<section class="panel project-panel">
  {#if !isCompactLayout}
    <div class="sort-row">
      <button
        class:sort-active={sortMode === "manual"}
        onclick={() => onSetSortMode("manual")}
      >
        Manual
      </button>
      <button
        class:sort-active={sortMode === "alphabetical"}
        onclick={() => onSetSortMode("alphabetical")}
      >
        A-Z
      </button>
      <button
        class:sort-active={sortMode === "recent"}
        onclick={() => onSetSortMode("recent")}
      >
        Recent
      </button>
    </div>
  {/if}
  <div class="project-list">
    {#if allProjects.length === 0}
      <div class="empty">No projects yet.</div>
    {/if}
    {#each allProjects as project}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class:active={activeProject === project}
        class:manual-sort={effectiveSortMode === "manual"}
        class:dragging={draggedProject === project}
        class:drop-target={dropTargetProject === project}
        class:drop-after={dropTargetProject === project &&
          dropPosition === "after"}
        class="project-row"
        onmousemove={(event) => onHandleDragOver(event, project)}
      >
        {#if effectiveSortMode === "manual"}
          <button
            class="drag-handle"
            aria-label="Reorder project"
            title="Drag to reorder"
            onmousedown={(event) => {
              event.preventDefault();
              event.stopPropagation();
              onHandleDragStart(project);
            }}
          >
            |||
          </button>
        {/if}
        <button class="project-button" onclick={() => onSelectProject(project)}>
          <span>{project}</span>
          {#if activeProject === project}<strong>Active</strong>{/if}
        </button>
        {#if permanentProjects.includes(project)}
          <button
            class="icon-button"
            title="Remove project"
            onclick={() => onRemoveProject(project)}
          >
            x
          </button>
        {:else}
          <button
            class="icon-button icon-add"
            title="Save project"
            onclick={() => onSaveAdhocProject(project)}
          >
            +
          </button>
        {/if}
      </div>
    {/each}
  </div>
</section>
