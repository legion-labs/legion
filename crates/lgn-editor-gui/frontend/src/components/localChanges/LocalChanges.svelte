<script lang="ts">
  import {
    stagedResources,
    stagedResourcesMode,
  } from "@/stores/stagedResources";
  import { PanelHeader } from "@lgn/web-client/src/components/panel";
  import LocalChangesGrid from "./LocalChangesGrid.svelte";
  import LocalChangesHeader from "./LocalChangesHeader.svelte";
  import LocalChangesList from "./LocalChangesList.svelte";
</script>

<div class="root">
  <PanelHeader>
    <LocalChangesHeader />
  </PanelHeader>

  <div class="content">
    {#if $stagedResources && $stagedResources.length}
      {#if $stagedResourcesMode === "card"}
        <LocalChangesGrid stagedResources={$stagedResources} />
      {:else if $stagedResourcesMode === "list"}
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
