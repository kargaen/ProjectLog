<script lang="ts">
  let {
    version,
    onOpenProjectHomepage,
    onOpenFeedback,
    onOpenPortfolio,
    onOpenDiagnosticLog,
    onClose,
  }: {
    version: string;
    onOpenProjectHomepage: () => void | Promise<void>;
    onOpenFeedback: () => void | Promise<void>;
    onOpenPortfolio: () => void | Promise<void>;
    onOpenDiagnosticLog: () => void | Promise<void>;
    onClose: () => void | Promise<void>;
  } = $props();
</script>

<div class="dialog-backdrop">
  <div class="dialog about-dialog">
    <h2>About ProjectLog</h2>
    <p class="about-copy">
      ProjectLog helps you track project changes with as little friction as
      possible.
    </p>
    <p class="about-meta">Version {version}</p>
    <p class="about-meta">Developed by Karsten Garborg.</p>
    <p class="about-links">
      <a
        href="https://github.com/kargaen/ProjectLog"
        onclick={(event) => {
          event.preventDefault();
          onOpenProjectHomepage();
        }}
      >
        ProjectLog
      </a>
      <span> | </span>
      <a
        href="mailto:karga@karga.dk"
        onclick={(event) => {
          event.preventDefault();
          onOpenFeedback();
        }}
      >
        Send feedback by mail
      </a>
      <span> | </span>
      <a
        href="https://karga.dk"
        onclick={(event) => {
          event.preventDefault();
          onOpenPortfolio();
        }}
      >
        Portfolio
      </a>
    </p>
    <p class="about-links">
      <a
        href="/diagnostic-log"
        onclick={(event) => {
          event.preventDefault();
          onOpenDiagnosticLog();
        }}
      >
        Open diagnostic log
      </a>
    </p>
    <div class="dialog-buttons about-buttons">
      <button class="primary" onclick={onClose}>Close</button>
    </div>
  </div>
</div>
