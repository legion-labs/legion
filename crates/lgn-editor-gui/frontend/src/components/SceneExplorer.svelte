<script lang="ts">
  import Icon from "@iconify/svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";

  import { resourceDragAndDropType } from "@/constants";
  import type { Entries, Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { iconFor } from "@/lib/resourceBrowser";
  import { fetchCurrentResourceDescription } from "@/orchestrators/currentResource";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";

  export let currentResourceDescriptionEntry: Entry<ResourceDescription> | null;

  export let resourceEntries: Entries<ResourceDescription>;

  export let allResourcesLoading: boolean;

  function selectResource({
    detail: resourceDescription,
  }: CustomEvent<Entry<ResourceDescription>>) {
    if (resourceDescription) {
      fetchCurrentResourceDescription(resourceDescription.item.id);
    }
  }
</script>

<Panel loading={allResourcesLoading} tabs={["Scene Explorer"]}>
  <div slot="tab" let:tab>{tab}</div>
  <div slot="content" class="content">
    {#if !resourceEntries.isEmpty()}
      <HierarchyTree
        id="scene-explorer"
        itemContextMenu="scene"
        draggable={resourceDragAndDropType}
        on:select={selectResource}
        bind:entries={resourceEntries}
        bind:highlightedEntry={currentResourceDescriptionEntry}
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
