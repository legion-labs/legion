<script lang="ts">
  import { fetchCurrentResourceDescription } from "@/stores/currentResource";
  import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import PanelList from "@lgn/web-client/src/components/panel/PanelList.svelte";
  import { Entries } from "@/lib/hierarchyTree";

  export let currentResourceDescription: ResourceDescription | null;

  export let resourceEntries: Entries<ResourceDescription | symbol>;

  export let allResourcesLoading: boolean;

  // This part is not well optimized but should be dropped eventually
  $: allResources = resourceEntries.intoItems().reduce((acc, resource) => {
    if (typeof resource === "symbol") {
      return acc;
    }

    return [...acc, resource];
  }, [] as ResourceDescription[]);
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
