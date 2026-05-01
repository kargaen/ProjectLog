<script lang="ts">
  type UpdateStatus = "idle" | "available" | "downloading" | "ready";

  let {
    updateStatus,
    updateVersion,
    updateProgress,
    onOpenReleaseNotes,
    onClose,
    onInstallUpdate,
  }: {
    updateStatus: UpdateStatus;
    updateVersion: string;
    updateProgress: number;
    onOpenReleaseNotes: () => void | Promise<void>;
    onClose: () => void | Promise<void>;
    onInstallUpdate: () => void | Promise<void>;
  } = $props();
</script>

<div class="dialog-backdrop">
  <div class="dialog about-dialog">
    <h2>ProjectLog update</h2>
    {#if updateStatus === "available"}
      <p class="about-copy">
        Version {updateVersion} is ready. ProjectLog can download and install
        it for you.
      </p>
      <p class="about-meta">Release notes open on the GitHub releases page.</p>
      <div class="dialog-buttons about-buttons">
        <button onclick={onOpenReleaseNotes}>Release notes</button>
        <button onclick={onClose}>Later</button>
        <button class="primary" onclick={onInstallUpdate}>Update now</button>
      </div>
    {:else if updateStatus === "downloading"}
      <p class="about-copy">Downloading and installing the update.</p>
      <p class="about-meta">{Math.round(updateProgress)}%</p>
    {:else if updateStatus === "ready"}
      <p class="about-copy">Update installed. ProjectLog is restarting.</p>
    {:else}
      <p class="about-copy">No update is ready right now.</p>
      <div class="dialog-buttons about-buttons">
        <button class="primary" onclick={onClose}>Close</button>
      </div>
    {/if}
  </div>
</div>
