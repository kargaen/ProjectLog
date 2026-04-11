<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let mode = $state("");
  let title = $state("");
  let value = $state("");
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    const unlisten = listen<{ mode: string; title: string; value: string }>(
      "show-input",
      (event) => {
        mode = event.payload.mode;
        title = event.payload.title;
        value = event.payload.value;
        setTimeout(() => inputEl?.focus(), 50);
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function submit() {
    await invoke("submit_input", { mode, value: value.trim() });
    await getCurrentWindow().hide();
    value = "";
  }

  async function cancel() {
    await getCurrentWindow().hide();
    value = "";
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") submit();
    if (e.key === "Escape") cancel();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main>
  <h3>{title || "Input"}</h3>
  <input
    bind:this={inputEl}
    bind:value
    type="text"
    placeholder="Type here..."
  />
  <div class="buttons">
    <button onclick={cancel}>Cancel</button>
    <button class="primary" onclick={submit}>OK</button>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #f5f5f5;
  }
  main {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  h3 {
    margin: 0;
    font-size: 14px;
    color: #333;
  }
  input {
    padding: 8px 12px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 14px;
    outline: none;
  }
  input:focus {
    border-color: #4285f4;
    box-shadow: 0 0 0 2px rgba(66, 133, 244, 0.2);
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  button {
    padding: 6px 16px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    font-size: 13px;
  }
  button:hover {
    background: #f0f0f0;
  }
  button.primary {
    background: #4285f4;
    color: white;
    border-color: #4285f4;
  }
  button.primary:hover {
    background: #3367d6;
  }
</style>
