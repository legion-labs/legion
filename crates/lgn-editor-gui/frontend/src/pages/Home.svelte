<script lang="ts">
  import { Panel, PanelList } from "@lgn/web-client/src/components/panel";
  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import ViewportPanel from "@lgn/web-client/src/components/panel/ViewportPanel.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import {
    getAllResources,
    getResourceProperties,
    removeResource,
    renameResource,
  } from "@/api";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import currentResourceStore from "@/stores/currentResource";
  import HierarchyTreeOrchestrator from "@/stores/hierarchyTree";
  import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import HierarchyTree from "@/components/hierarchyTree/HierarchyTree.svelte";
  import log from "@lgn/web-client/src/lib/log";
  import { Entry } from "@/lib/hierarchyTree";
  import contextMenu from "@/actions/contextMenu";
  import contextMenuStore, {
    ContextMenuEntryRecord,
  } from "@/stores/contextMenu";
  import * as contextMenuEntries from "@/data/contextMenu";
  import {
    autoClose,
    Event as ContextMenuActionEvent,
    select,
  } from "@lgn/web-client/src/types/contextMenu";
  import ModalStore from "@lgn/web-client/src/stores/modal";
  import CreateResourceModal from "@/components/resources/CreateResourceModal.svelte";
  import { SvelteComponent } from "svelte";
  import allResourcesStore from "@/stores/allResources";
  import viewportOrchestrator from "@/stores/viewport";
  import notificationsStore from "@/stores/notifications";
  import { components, join } from "@/lib/path";

  contextMenuStore.register("resource", contextMenuEntries.resourceEntries);
  contextMenuStore.register(
    "resourcePanel",
    contextMenuEntries.resourcePanelEntries
  );

  const editorViewportKey = Symbol();

  viewportOrchestrator.addAllViewport(
    [editorViewportKey, { type: "video", name: "editor" }],
    [Symbol(), { type: "video", name: "runtime" }]
  );

  viewportOrchestrator.activate(editorViewportKey);

  const modalStore = new ModalStore();

  const { data: currentResourceData } = currentResourceStore;

  const createResourceModalId = Symbol();

  const resourceEntriesOrchestrator =
    new HierarchyTreeOrchestrator<ResourceDescription>();

  const {
    entries: resourceEntriesStore,
    currentlyRenameEntry: currentlyRenameResourceStore,
  } = resourceEntriesOrchestrator;

  let allResourcesData = allResourcesStore.data;

  let currentResourceDescription: ResourceDescription | null = null;

  let allResourcesPromise = allResourcesStore.run(getAllResources);

  let resourceHierarchyTree: HierarchyTree<ResourceDescription> | null = null;

  $: if ($allResourcesData) {
    resourceEntriesOrchestrator.load($allResourcesData);
  }

  function fetchCurrentResourceDescription() {
    if (!currentResourceDescription) {
      return;
    }

    try {
      currentResourceStore.run(() => {
        if (!currentResourceDescription) {
          throw new Error("Current resource description not found");
        }

        return getResourceProperties(currentResourceDescription);
      });
    } catch (error) {
      notificationsStore.push(Symbol(), {
        type: "error",
        title: "Resources",
        message: "An error occured while loading the resource",
      });

      log.error(
        log.json`An error occured while loading the resource ${currentResourceDescription}: ${error}`
      );
    }
  }

  async function saveEditedResourceProperty({
    detail: { entry, newName },
  }: CustomEvent<{
    entry: Entry<ResourceDescription | symbol>;
    newName: string;
  }>) {
    if (typeof entry.item === "symbol") {
      return;
    }

    const pathComponents = components(entry.item.path);

    if (!pathComponents) {
      return;
    }

    const newPath = join([...pathComponents.slice(0, -1), newName]);

    try {
      await renameResource({ id: entry.item.id, newPath });
    } catch (error) {
      notificationsStore.push(Symbol(), {
        type: "error",
        title: "Resources",
        message: "An error occured while renaming the resource",
      });

      log.error(
        log.json`An error occured while renaming the resource ${entry.item}: ${error}`
      );
    }
  }

  async function removeResourceProperty({
    detail: entry,
  }: CustomEvent<Entry<ResourceDescription | symbol>>) {
    if (typeof entry.item === "symbol") {
      return;
    }

    try {
      await removeResource({ id: entry.item.id });
    } catch (error) {
      notificationsStore.push(Symbol(), {
        type: "error",
        title: "Resources",
        message: "An error occured while removing the resource",
      });

      log.error(
        log.json`An error occured while removing the resource ${entry.item}: ${error}`
      );
    }
  }

  function tryAgain() {
    $currentResourceData = null;
    currentResourceDescription = null;
    allResourcesPromise = allResourcesStore.run(getAllResources);
  }

  async function handleResourceRename({
    detail: { action, entrySetName },
  }: ContextMenuActionEvent<
    Pick<ContextMenuEntryRecord, "resource" | "resourcePanel">
  >) {
    switch (action) {
      case "rename": {
        if (!resourceHierarchyTree || !currentResourceDescription) {
          return;
        }

        resourceHierarchyTree.startNameEdit(currentResourceDescription);

        return;
      }

      case "remove": {
        if (!resourceHierarchyTree || !currentResourceDescription) {
          return;
        }

        resourceHierarchyTree.remove(currentResourceDescription);

        return;
      }

      case "new": {
        modalStore.open(
          createResourceModalId,
          // TODO: Fix the typings
          CreateResourceModal as unknown as SvelteComponent,
          entrySetName === "resource" ? currentResourceDescription : null
        );
      }

      default: {
        return;
      }
    }
  }
</script>

<ModalContainer store={modalStore} />

<ContextMenu store={contextMenuStore} />

<Notifications store={notificationsStore} />

<svelte:window
  on:contextmenu-action={autoClose(
    select(handleResourceRename, "resource", "resourcePanel")
  )}
/>

<div class="root">
  <TopBar />
  <div class="content-wrapper">
    <div class="content">
      <div class="secondary-contents">
        <div class="scene-explorer">
          <Panel let:isFocused tabs={["Scene Explorer"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div slot="content" class="scene-explorer-content">
              {#await allResourcesPromise}
                <div class="scene-explorer-loading">Loading...</div>
              {:then resources}
                <PanelList
                  key="id"
                  items={resources}
                  panelIsFocused={isFocused}
                  on:select={fetchCurrentResourceDescription}
                  bind:highlightedItem={currentResourceDescription}
                >
                  <div slot="default" let:item={resource}>
                    {resource.path}
                  </div>
                </PanelList>
              {:catch}
                <div class="scene-explorer-error">
                  An error occured while fetching the scene explorer
                  <span class="scene-explorer-try-again" on:click={tryAgain}>
                    try again
                  </span>
                </div>
              {/await}
            </div>
          </Panel>
        </div>
        <div class="h-separator" />
        <div class="resource-browser">
          <Panel tabs={["Resource Browser"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div
              slot="content"
              class="resource-browser-content"
              use:contextMenu={"resourcePanel"}
            >
              {#if $allResourcesData}
                <!-- Works as expected, the type error seems to be related to Svelte
                     https://github.com/sveltejs/svelte/issues/7225 -->
                <HierarchyTree
                  on:select={fetchCurrentResourceDescription}
                  on:nameEdited={saveEditedResourceProperty}
                  on:removed={removeResourceProperty}
                  bind:entries={$resourceEntriesStore}
                  bind:currentlyRenameEntry={$currentlyRenameResourceStore}
                  bind:highlightedItem={currentResourceDescription}
                  bind:this={resourceHierarchyTree}
                >
                  <div
                    class="h-full w-full"
                    slot="name"
                    use:contextMenu={"resource"}
                    let:itemName
                  >
                    {itemName}
                  </div>
                </HierarchyTree>
              {/if}
            </div>
          </Panel>
        </div>
      </div>
      <div class="v-separator" />
      <div class="main-content">
        <ViewportPanel orchestrator={viewportOrchestrator} />
      </div>
      <div class="v-separator" />
      <div class="secondary-contents">
        <div class="property-grid">
          <Panel tabs={["Property Grid"]}>
            <div slot="tab" let:tab>
              {tab}
            </div>
            <div class="property-grid-content" slot="content">
              <PropertyGrid />
            </div>
          </Panel>
        </div>
      </div>
    </div>
  </div>
  <StatusBar />
</div>

<style lang="postcss">
  .root {
    @apply h-screen w-full;

    .content-wrapper {
      @apply h-[calc(100vh-4rem)] w-full overflow-auto;
    }
  }

  .content {
    @apply flex flex-row h-full w-full;
  }

  .main-content {
    @apply flex flex-col w-full;
  }

  .v-separator {
    @apply flex-shrink-0 w-1;
  }

  .h-separator {
    @apply flex-shrink-0 h-1;
  }

  .secondary-contents {
    @apply flex flex-col flex-shrink-0 w-96 h-full;
  }

  .scene-explorer {
    @apply h-[calc(50%-theme("spacing[0.5]"))];
  }

  .scene-explorer-loading {
    @apply px-2 py-1;
  }

  .scene-explorer-error {
    @apply px-2 py-1;
  }

  .scene-explorer-try-again {
    @apply underline text-blue-300 cursor-pointer;
  }

  .scene-explorer-content {
    @apply h-full break-all;
  }

  .resource-browser {
    @apply h-[calc(50%-theme("spacing[0.5]"))];
  }

  .resource-browser-content {
    @apply h-full;
  }

  .property-grid {
    @apply h-full;
  }

  .property-grid-content {
    @apply h-full;
  }
</style>
