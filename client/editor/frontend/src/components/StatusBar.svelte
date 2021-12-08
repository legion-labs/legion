<script lang="ts">
  import statusBar from "@/stores/statusBar";
</script>

<div class="root">
  <div class="status-bar">
    <span class="status">
      {#if $statusBar}
        {$statusBar}
      {/if}
    </span>
  </div>
</div>

<style lang="postcss">
  .status {
    animation: glow 1s infinite alternate;
  }
  @keyframes glow {
    from {
      opacity: 0.2;
    }

    to {
      opacity: 1;
    }
  }
</style>
