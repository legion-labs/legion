<script lang="ts">
  import { ServerType } from "@lgn/frontend/src/api";
  import { Resolution } from "@lgn/frontend/src/lib/types";
  import { Panel, PanelList } from "@lgn/frontend/src/components/panel";
  import ContextMenu from "@lgn/frontend/src/components/ContextMenu.svelte";
  import TopBar from "@lgn/frontend/src/components/TopBar.svelte";
  import StatusBar from "@lgn/frontend/src/components/StatusBar.svelte";
  import RemoteWindow from "@lgn/frontend/src/components/RemoteWindow.svelte";
  import { getAllResources, getResourceProperties } from "@/api";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import currentResource from "@/stores/currentResource";
  import { ResourceDescription } from "@lgn/proto-editor/dist/editor";
  import ScriptEditor from "@/components/ScriptEditor.svelte";
  import HierarchyTree from "@/components/hierarchyTree/HierarchyTree.svelte";
  import log from "@lgn/frontend/src/lib/log";
  import { Entries } from "@/lib/hierarchyTree";
  import { AsyncStoreOrchestratorList } from "@lgn/frontend/src/stores/asyncStore";
  import contextMenu from "@/actions/contextMenu";
  import contextMenuStore, {
    ContextMenuEntryRecord,
  } from "@/stores/contextMenu";
  import contextMenuEntries from "@/data/contextMenu";
  import {
    autoClose,
    Event as ContextMenuActionEvent,
    select,
  } from "@lgn/frontend/src/types/contextMenu";

  contextMenuStore.register("resource", contextMenuEntries);

  const { data: currentResourceData } = currentResource;

  const allResourcesStore = new AsyncStoreOrchestratorList<
    ResourceDescription[]
  >();

  let allResourcesData = allResourcesStore.data;

  let currentResourceDescription: ResourceDescription | null = null;

  let desiredVideoResolution: Resolution | null;

  let editorActiveTab: ServerType;

  let allResourcesPromise = allResourcesStore.run(getAllResources);

  let resourceHierarchyTree: HierarchyTree<ResourceDescription> | null = null;

  function fetchCurrentResourceDescription() {
    if (!currentResourceDescription) {
      return;
    }

    try {
      currentResource.run(() => {
        if (!currentResourceDescription) {
          throw new Error("Current resource description not found");
        }

        return getResourceProperties(currentResourceDescription);
      });
    } catch (error) {
      log.error(
        log.json`An error occured while loading the resource ${currentResourceDescription}: ${error}`
      );
    }
  }

  function tryAgain() {
    $currentResourceData = null;
    currentResourceDescription = null;
    allResourcesPromise = allResourcesStore.run(getAllResources);
  }

  function handleResourceRename({
    detail: { action },
  }: ContextMenuActionEvent<Pick<ContextMenuEntryRecord, "resource">>) {
    switch (action) {
      case "rename": {
        if (!currentResourceDescription || !resourceHierarchyTree) {
          return;
        }

        resourceHierarchyTree.edit(currentResourceDescription);

        return;
      }

      default: {
        return;
      }
    }
  }
</script>

<ContextMenu {contextMenuStore} />

<svelte:window
  on:contextmenu-action={autoClose(select(handleResourceRename, "resource"))}
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
                  bind:selectedItem={currentResourceDescription}
                  panelIsFocused={isFocused}
                  on:dblclick={fetchCurrentResourceDescription}
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
          <Panel let:isFocused tabs={["Resource Browser"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div slot="content" class="resource-browser-content">
              {#if $allResourcesData}
                <HierarchyTree
                  entries={Entries.unflatten($allResourcesData, Symbol)}
                  panelIsFocused={isFocused}
                  on:dblclick={fetchCurrentResourceDescription}
                  bind:selectedItem={currentResourceDescription}
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
        <Panel
          tabs={["editor", "runtime", "script"]}
          bind:activeTab={editorActiveTab}
        >
          <div slot="tab" let:tab>
            {#if tab === "editor" || tab === "runtime"}
              <span>{tab[0].toUpperCase()}{tab.slice(1)}</span>
              {#if desiredVideoResolution}
                <span>
                  - {desiredVideoResolution.width}x{desiredVideoResolution.height}
                </span>
              {/if}
            {:else if tab === "script"}
              Script
            {/if}
          </div>
          <div class="video-container" slot="content">
            {#if editorActiveTab === "editor" || editorActiveTab === "runtime"}
              {#key editorActiveTab}
                <RemoteWindow
                  serverType={editorActiveTab}
                  bind:desiredResolution={desiredVideoResolution}
                />
              {/key}
            {:else if editorActiveTab === "script"}
              <ScriptEditor theme="vs-dark" />
            {/if}
          </div>
        </Panel>
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

  .video-container {
    @apply h-full w-full;
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
