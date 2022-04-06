<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onDestroy } from "svelte";
  import { readable } from "svelte/store";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import type { ContextMenuEvent } from "@lgn/web-client/src/types/contextMenu";
  import { filterContextMenuEvents } from "@lgn/web-client/src/types/contextMenu";

  import { closeScene } from "@/api";
  import { resourceDragAndDropType } from "@/constants";
  import type { Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { iconFor } from "@/lib/resourceBrowser";
  import { fetchAllActiveScenes } from "@/orchestrators/allActiveScenes";
  import { fetchCurrentResourceDescription } from "@/orchestrators/currentResource";
  import { deriveHierarchyTreeOrchestrator } from "@/orchestrators/hierarchyTree";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import { sceneExplorerItemContextMenuId } from "@/stores/contextMenu";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";

  export let activeScenes: ResourceDescription[];

  export let rootScene: ResourceDescription;

  $: sceneExplorerEntriesOrchestrator = deriveHierarchyTreeOrchestrator(
    readable(activeScenes)
  );

  $: currentSceneDescriptionEntry =
    sceneExplorerEntriesOrchestrator.currentEntry;

  $: sceneEntries = sceneExplorerEntriesOrchestrator.entries;

  onDestroy(() => {
    sceneExplorerEntriesOrchestrator.unsubscriber();
  });

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
        await closeScene({ id: rootScene.id });

        await fetchAllActiveScenes();

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

<div class="root">
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

<style lang="postcss">
  .root {
    @apply h-full break-all;
  }
</style>
