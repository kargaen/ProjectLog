<script lang="ts">
  let {
    appIcon,
    activeProject,
    activeComment,
    showActiveComment,
    onStartDrag,
    onHide,
  }: {
    appIcon: string;
    activeProject: string;
    activeComment: string;
    showActiveComment: boolean;
    onStartDrag: () => void | Promise<void>;
    onHide: () => void | Promise<void>;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="quickpanel-header">
  <div class="quickpanel-drag" onmousedown={onStartDrag}>
    <img class="logo" src={appIcon} alt="ProjectLog" />
    <div>
      <h1>ProjectLog QuickPanel</h1>
      <p>{activeProject || "No active project"}</p>
      {#if showActiveComment && activeComment}
        <p class="active-comment">{activeComment}</p>
      {/if}
    </div>
  </div>
  <div class="header-actions">
    <button
      class="ghost"
      onmousedown={(event) => event.stopPropagation()}
      onclick={onHide}
    >
      Hide
    </button>
  </div>
</header>
