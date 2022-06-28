<script lang="ts">
  import type { SourceControl } from "@lgn/api/editor";

  import statusStore from "../stores/statusBarData";
  import Button from "./Button.svelte";
  import PingPoint from "./PingPoint.svelte";

  /**
   * An array containing all the staged resources (if any).
   * If the value is `null` the whole local storage section will be hidden.
   */
  export let stagedResources: SourceControl.StagedResource[] | null = null;

  export let syncFromMain: (() => void) | null = null;
</script>

<div class="root">
  <div class="status">
    <div class="status-message">
      {#if $statusStore}
        {$statusStore}
      {/if}
    </div>
  </div>
  {#if stagedResources}
    <div class="local-changes">
      {#if stagedResources.length}
        <div class="ping-point">
          <PingPoint title="Local changes detected" />
        </div>
        <div>Local changes detected</div>
      {:else}
        <div class="ping-point">
          <PingPoint disabled title="No local changes detected" />
        </div>
        <div>No local changes detected</div>
      {/if}
      {#if syncFromMain}
        <div>
          <Button variant="success" on:click={syncFromMain}>
            Sync from main
          </Button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row justify-between px-2 h-10 items-center;
  }

  .status {
    @apply flex flex-row items-center h-full;
  }

  .status-message {
    @apply flex flex-row items-center h-full animate-pulse;
  }

  .local-changes {
    @apply flex flex-row items-center h-full space-x-2;
  }

  .ping-point {
    @apply h-3 w-3;
  }
</style>
