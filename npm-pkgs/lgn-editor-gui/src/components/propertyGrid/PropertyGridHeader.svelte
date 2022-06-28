<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Writable } from "svelte/store";

  import MenuBar from "@lgn/web-client/src/components/menu/MenuBar.svelte";

  import type {
    ResourceProperty,
    ResourceWithProperties,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import { propertiesAreEntities } from "@/components/propertyGrid/lib/propertyGrid";
  import modal from "@/stores/modal";

  import CreateComponentModal from "../resources/CreateComponentModal.svelte";

  const componentKey = "components";

  export let resources: ResourceWithProperties[];
  export let search: Writable<string>;

  let div: HTMLElement;

  function onAddComponentClicked() {
    // Since we dont' support multi edition for now we pick the first entity
    const resource = resources[0];

    modal.open(Symbol(), CreateComponentModal, {
      payload: {
        resourceId: resource.description.id,
        path: componentKey,
        index: getComponentProperty(resource.properties)?.sub_properties.length,
      },
    });
  }

  function getComponentProperty(
    properties: ResourceProperty[]
  ): ResourceProperty | null {
    properties.forEach((p) => {
      const c = p.sub_properties.find((c) => c.name === componentKey);

      if (c) {
        return c;
      }

      return getComponentProperty(p.sub_properties);
    });

    return null;
  }
</script>

<div class="property-header-root" bind:this={div}>
  <div class="header-root">
    <div class="header-text">Properties</div>
    <div class="action">
      <MenuBar
        container={div}
        enableHover={false}
        items={[
          {
            icon: "ic:outline-more-vert",
            visible: true,
            children: [
              {
                visible: resources.length >= 2,
                title: "Copy Ids",
                action: async () => {
                  await navigator.clipboard.writeText(
                    resources.map((r) => r.id).join(";")
                  );
                },
              },
              {
                visible: resources.length === 1,
                title: "Copy Id",
                action: async () => {
                  await navigator.clipboard.writeText(resources[0].id);
                },
              },
            ],
          },
        ]}
      />
    </div>
  </div>
  <div class="search">
    <input type="text" placeholder="Search property" bind:value={$search} />
    <div class="icon">
      <Icon class="text-item-low" icon="ic:sharp-search" />
    </div>
  </div>
  {#if propertiesAreEntities(resources)}
    <button on:click={onAddComponentClicked}>+ Add component</button>
  {/if}
</div>

<style lang="postcss">
  .property-header-root {
    @apply bg-surface-600 p-2 gap-y-2 flex flex-col;
  }

  .header-root {
    @apply flex place-content-between;
  }

  input {
    @apply w-full h-full rounded-sm;
  }

  .header-text {
    @apply font-semibold text-item-high;
  }

  .action {
    @apply text-item-low cursor-pointer self-center;
  }

  .search {
    @apply relative h-7;
  }

  .icon {
    @apply absolute right-1 top-1/4;
  }

  input {
    @apply bg-surface-max pl-2;
  }

  button {
    @apply w-full bg-surface-500 rounded-sm border border-content-max text-item-low;
  }
</style>
