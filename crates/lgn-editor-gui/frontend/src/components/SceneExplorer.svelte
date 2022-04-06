<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onDestroy } from "svelte";
  import { readable } from "svelte/store";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

  import { resourceDragAndDropType } from "@/constants";
  import type { Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { iconFor } from "@/lib/resourceBrowser";
  import { fetchCurrentResourceDescription } from "@/orchestrators/currentResource";
  import { deriveHierarchyTreeOrchestrator } from "@/orchestrators/hierarchyTree";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";

  export let activeScenes: ResourceDescription[];

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
</script>

<div class="root">
  {#if !$sceneEntries.isEmpty()}
    <HierarchyTree
      id="scene-explorer"
      draggable={resourceDragAndDropType}
      on:select={selectResource}
      bind:entries={$sceneEntries}
      bind:highlightedEntry={$currentSceneDescriptionEntry}
    >
      <div class="entry" slot="entry" let:entry let:isHighlighted>
        <div
          class="entry-icon"
          class:text-gray-400={!isHighlighted}
          class:text-orange-700={isHighlighted}
        >
          <Icon class="w-full h-full" icon={iconFor(entry)} />
        </div>
        <div class="entry-name" title={isEntry(entry) ? entry.item.path : null}>
          {entry.name}
        </div>
      </div>
    </HierarchyTree>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply h-full break-all;
  }

  .entry {
    @apply flex flex-row w-full h-full space-x-1;
  }

  .entry-icon {
    @apply w-6 h-6;
  }

  .entry-name {
    @apply w-full h-full;
  }
</style>
