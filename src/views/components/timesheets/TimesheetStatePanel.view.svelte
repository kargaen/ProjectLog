<script lang="ts">
  let {
    title,
    body,
  }: {
    title: string;
    body: string;
  } = $props();
</script>

<section class="timesheet-preview-panel timesheet-preview-loading">
  <h1>{title}</h1>
  <p>{body}</p>
</section>
