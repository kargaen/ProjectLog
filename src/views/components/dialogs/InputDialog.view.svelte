<script lang="ts">
  let {
    title,
    value,
    onValueChange,
    onCancel,
    onSubmit,
  }: {
    title: string;
    value: string;
    onValueChange: (value: string) => void;
    onCancel: () => void | Promise<void>;
    onSubmit: () => void | Promise<void>;
  } = $props();
</script>

<div class="dialog-backdrop">
  <div class="dialog">
    <h2>{title || "Input"}</h2>
    <input
      class="dialog-input"
      type="text"
      value={value}
      placeholder="Type here..."
      oninput={(event) =>
        onValueChange((event.currentTarget as HTMLInputElement).value)}
    />
    <div class="dialog-buttons">
      <button onclick={onCancel}>Cancel</button>
      <button class="primary" onclick={onSubmit}>OK</button>
    </div>
  </div>
</div>
