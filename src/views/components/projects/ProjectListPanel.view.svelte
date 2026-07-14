<script lang="ts">
  import type { SortMode } from "../../../models/types";
  import type { ProjectGroupBucket } from "../../../controllers/quickpanel/quickPanelTypes";
  import ProjectContextMenu from "./ProjectContextMenu.view.svelte";

  let {
    isCompactLayout,
    sortMode,
    allProjects,
    groupedProjects,
    groupProjectsEnabled,
    knownGroupNames,
    projectColors,
    projectGroups,
    activeProject,
    permanentProjects,
    effectiveSortMode,
    draggedProject,
    dropTargetProject,
    dropPosition,
    contextMenuProject,
    contextMenuPosition,
    onSetSortMode,
    onToggleGroupProjectsEnabled,
    onHandleDragOver,
    onHandleDragStart,
    onSelectProject,
    onRemoveProject,
    onSaveAdhocProject,
    onOpenContextMenu,
    onCloseContextMenu,
    onPickColor,
    onPickGroup,
    onRequestNewGroup,
  }: {
    isCompactLayout: boolean;
    sortMode: SortMode;
    allProjects: string[];
    groupedProjects: ProjectGroupBucket[];
    groupProjectsEnabled: boolean;
    knownGroupNames: string[];
    projectColors: Record<string, string>;
    projectGroups: Record<string, string>;
    activeProject: string;
    permanentProjects: string[];
    effectiveSortMode: SortMode;
    draggedProject: string | null;
    dropTargetProject: string | null;
    dropPosition: "before" | "after";
    contextMenuProject: string | null;
    contextMenuPosition: { x: number; y: number } | null;
    onSetSortMode: (mode: SortMode) => void;
    onToggleGroupProjectsEnabled: () => void;
    onHandleDragOver: (event: MouseEvent, project: string) => void;
    onHandleDragStart: (project: string) => void;
    onSelectProject: (project: string) => void | Promise<void>;
    onRemoveProject: (project: string) => void | Promise<void>;
    onSaveAdhocProject: (project: string) => void | Promise<void>;
    onOpenContextMenu: (project: string, x: number, y: number) => void;
    onCloseContextMenu: () => void;
    onPickColor: (color: string | null) => void | Promise<void>;
    onPickGroup: (group: string | null) => void | Promise<void>;
    onRequestNewGroup: () => void;
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
      <button
        class:sort-active={groupProjectsEnabled}
        onclick={onToggleGroupProjectsEnabled}
      >
        Group
      </button>
    </div>
  {/if}
  <div class="project-list">
    {#if allProjects.length === 0}
      <div class="empty">No projects yet.</div>
    {/if}
    {#each groupedProjects as group}
      {#if group.groupName}
        <div class="group-header">{group.groupName}</div>
      {/if}
      {#each group.projects as project}
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
          oncontextmenu={(event) => {
            event.preventDefault();
            onOpenContextMenu(project, event.clientX, event.clientY);
          }}
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
          <button
            class="project-button"
            class:has-color={projectColors[project]}
            style={projectColors[project]
              ? `background: color-mix(in srgb, ${projectColors[project]}, transparent 60%); /* TODO: Quick fix - applies 40% opacity to background color to soften it. Adjust '60%' up to make it more transparent. */`
              : ""}
            onclick={() => onSelectProject(project)}
          >
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
    {/each}
  </div>
</section>

{#if contextMenuProject && contextMenuPosition}
  <ProjectContextMenu
    x={contextMenuPosition.x}
    y={contextMenuPosition.y}
    currentColor={projectColors[contextMenuProject] ?? null}
    currentGroup={projectGroups[contextMenuProject] ?? null}
    {knownGroupNames}
    onPickColor={onPickColor}
    onPickGroup={onPickGroup}
    onRequestNewGroup={onRequestNewGroup}
    onClose={onCloseContextMenu}
  />
{/if}
