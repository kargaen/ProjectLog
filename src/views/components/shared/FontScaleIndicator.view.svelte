<script lang="ts">
  type Props = {
    visible: boolean;
    scale: number;
  };

  const { visible, scale }: Props = $props();
</script>

{#if visible}
  <div class="font-scale-indicator" aria-hidden="true">
    {Math.round(scale * 100)}%
  </div>
{/if}

<style>
  .font-scale-indicator {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    padding: 10px 18px;
    background: rgba(32, 36, 31, 0.72);
    color: #fff;
    border-radius: 8px;
    font-size: 18px;
    font-weight: 650;
    pointer-events: none;
    z-index: 9999;
    animation: fade-in-out 1.2s ease forwards;
  }

  @keyframes fade-in-out {
    0% { opacity: 0; }
    15% { opacity: 1; }
    70% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
