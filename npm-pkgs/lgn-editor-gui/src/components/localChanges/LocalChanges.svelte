<script lang="ts">
  import { PanelHeader } from "@lgn/web-client/src/components/panel";
  import { filterContextMenuEvents } from "@lgn/web-client/src/types/contextMenu";
  import type { ContextMenuEvent } from "@lgn/web-client/src/types/contextMenu";

  import { revertResources } from "@/api";
  import { fetchAllResources } from "@/orchestrators/allResources";
  import { localChangesContextMenuId } from "@/stores/contextMenu";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import { stagedResourcesMode } from "@/stores/stagedResources";
  import { stagedResources } from "@/stores/stagedResources";

  import LocalChangesGrid from "./LocalChangesGrid.svelte";
  import LocalChangesHeader from "./LocalChangesHeader.svelte";
  import LocalChangesList from "./LocalChangesList.svelte";
  import { selectedLocalChange } from "./localChangesStore";

  async function handleContextMenuEvents({
    detail: { action, close },
  }: ContextMenuEvent<
    typeof localChangesContextMenuId,
    Pick<ContextMenuEntryRecord, typeof localChangesContextMenuId>
  >) {
    close();

    switch (action) {
      case "revert": {
        if ($selectedLocalChange?.info?.id) {
          await revertResources({
            ids: [$selectedLocalChange.info.id],
          });
          await fetchAllResources();
          break;
        }
      }
    }
  }
</script>

<svelte:window
  on:contextmenu-action={filterContextMenuEvents(
    handleContextMenuEvents,
    localChangesContextMenuId
  )}
/>

<div class="root">
  <PanelHeader>
    <LocalChangesHeader />
  </PanelHeader>

  <div class="content">
    {#if $stagedResources && $stagedResources.length}
      {#if $stagedResourcesMode === "card"}
        <LocalChangesGrid
          stagedResources={$stagedResources}
          selectedResource={selectedLocalChange}
        />
      {:else if $stagedResourcesMode === "list"}
        <LocalChangesList
          stagedResources={$stagedResources}
          selectedResource={selectedLocalChange}
        />
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
