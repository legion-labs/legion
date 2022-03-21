<script lang="ts">
  import { stagedResources } from "@/stores/stagedResources";
  import { PanelHeader } from "@lgn/web-client/src/components/panel";
  import LocalChangesGrid from "./LocalChangesGrid.svelte";
  import type { Mode } from "./LocalChangesHeader.svelte";
  import LocalChangesHeader from "./LocalChangesHeader.svelte";
  import LocalChangesList from "./LocalChangesList.svelte";

  let mode: Mode | undefined;
</script>

<div class="root">
  <PanelHeader>
    <LocalChangesHeader bind:mode />
  </PanelHeader>

  <div class="content">
    {#if $stagedResources && $stagedResources.length && mode}
      {#if mode === "card"}
        <LocalChangesGrid stagedResources={$stagedResources} />
      {:else if mode === "list"}
        <LocalChangesList stagedResources={$stagedResources} />
      {/if}
    {:else}
      <div class="no-local-changes">
        <em>No local changes</em>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-full w-full flex flex-col;
  }

  .content {
    @apply flex flex-col flex-grow overflow-hidden;
  }

  .no-local-changes {
    @apply flex justify-center items-center h-full w-full text-xl font-bold;
  }
</style>
