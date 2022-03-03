<script lang="ts">
  import { fetchCurrentResourceDescription } from "@/stores/currentResource";
  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import PanelList from "@lgn/web-client/src/components/panel/PanelList.svelte";
  import type { Entries } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{
    currentResourceDescriptionChange: ResourceDescription;
  }>();

  export let currentResourceDescription: ResourceDescription | null;

  export let resourceEntries: Entries<ResourceDescription>;

  export let allResourcesLoading: boolean;

  $: allResources = resourceEntries.intoItems();
</script>

<Panel loading={allResourcesLoading} tabs={["Scene Explorer"]}>
  <div slot="tab" let:tab>{tab}</div>
  <div slot="content" class="content" let:isFocused>
    {#if allResources.length > 0}
      <PanelList
        key="id"
        items={allResources}
        panelIsFocused={isFocused}
        on:select={({ detail: resourceDescription }) =>
          resourceDescription &&
          fetchCurrentResourceDescription(resourceDescription)}
        on:highlight={({ detail: item }) =>
          dispatch("currentResourceDescriptionChange", item)}
        bind:highlightedItem={currentResourceDescription}
      >
        <div slot="default" let:item={resource}>
          {resource.path}
        </div>
      </PanelList>
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .content {
    @apply h-full break-all;
  }
</style>
