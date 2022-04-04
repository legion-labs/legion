<script lang="ts">
  import Icon from "@iconify/svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import type { ContextMenuEvent } from "@lgn/web-client/src/types/contextMenu";
  import { filterContextMenuEvents } from "@lgn/web-client/src/types/contextMenu";

  import { closeScene } from "@/api";
  import { resourceDragAndDropType } from "@/constants";
  import type { Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { iconFor } from "@/lib/resourceBrowser";
  import { allActiveScenesLoading } from "@/orchestrators/allActiveScenes";
  import { fetchCurrentResourceDescription } from "@/orchestrators/currentResource";
  import {
    currentSceneDescriptionEntry,
    sceneEntries,
  } from "@/orchestrators/sceneExplorerEntries";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import { sceneExplorerItemContextMenuId } from "@/stores/contextMenu";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";

  function selectResource({
    detail: resourceDescription,
  }: CustomEvent<Entry<ResourceDescription>>) {
    if (resourceDescription) {
      fetchCurrentResourceDescription(resourceDescription.item.id);
    }
  }

  async function handleContextMenuEvents({
    detail: { action, close },
  }: ContextMenuEvent<
    typeof sceneExplorerItemContextMenuId,
    Pick<ContextMenuEntryRecord, typeof sceneExplorerItemContextMenuId>
  >) {
    close();

    switch (action) {
      case "closeScene": {
        if ($currentSceneDescriptionEntry) {
          await closeScene({ id: $currentSceneDescriptionEntry?.item.id });
        }

        return;
      }
    }
  }
</script>

<svelte:window
  on:contextmenu-action={filterContextMenuEvents(
    handleContextMenuEvents,
    sceneExplorerItemContextMenuId
  )}
/>

<Panel loading={$allActiveScenesLoading} tabs={["Scene Explorer"]}>
  <div slot="tab" let:tab>{tab}</div>
  <div slot="content" class="content">
    {#if !$sceneEntries.isEmpty()}
      <HierarchyTree
        id="scene-explorer"
        itemContextMenu={sceneExplorerItemContextMenuId}
        draggable={resourceDragAndDropType}
        on:select={selectResource}
        bind:entries={$sceneEntries}
        bind:highlightedEntry={$currentSceneDescriptionEntry}
      >
        <div class="w-full h-full" slot="icon" let:entry>
          <Icon class="w-full h-full" icon={iconFor(entry)} />
        </div>
        <div
          class="item"
          slot="name"
          let:entry
          title={isEntry(entry) ? entry.item.path : null}
        >
          {entry.name}
        </div>
      </HierarchyTree>
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .content {
    @apply h-full break-all;
  }
</style>
