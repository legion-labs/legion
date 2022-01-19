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
  import { unflatten } from "@/lib/hierarchyTree";
  import asyncStore from "@lgn/frontend/src/stores/asyncStore";
  import contextMenu from "@/actions/contextMenu";
  import contextMenuStore from "@/stores/contextMenu";
  import contextMenuEntries from "@/data/contextMenu";
  import { fakeFileSystemEntries } from "@/data/fake";

  contextMenuStore.register("resource", contextMenuEntries);

  const { data: currentResourceData } = currentResource;

  const allResourcesStore = asyncStore<ResourceDescription[]>();

  let allResourcesData = allResourcesStore.data;

  let currentResourceDescription: ResourceDescription | null = null;

  let desiredVideoResolution: Resolution | null;

  let editorActiveTab: ServerType;

  let allResourcesPromise = allResourcesStore.run(getAllResources);

  $: if (currentResourceDescription) {
    currentResource
      .run(() => {
        if (currentResourceDescription) {
          return getResourceProperties(currentResourceDescription);
        } else {
          throw new Error("Current resource description not found");
        }
      })
      .catch((error) =>
        log.error(
          log.json`An error occured while loading the resource ${currentResourceDescription}: ${error}`
        )
      );
  }

  function tryAgain() {
    $currentResourceData = null;
    currentResourceDescription = null;
    allResourcesPromise = allResourcesStore.run(getAllResources);
  }

  function setCurrentResourceDescription(
    resourceDescription: ResourceDescription
  ) {
    currentResourceDescription = resourceDescription;
  }
</script>

<ContextMenu {contextMenuStore} />

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
                  activeItem={currentResourceDescription}
                  panelIsFocused={isFocused}
                  on:click={({ detail: resource }) =>
                    setCurrentResourceDescription(resource)}
                  on:itemChange={({ detail: { newItem: resource } }) =>
                    setCurrentResourceDescription(resource)}
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
            <div slot="content" class="resource-browser-content">
              {#if $allResourcesData}
                <HierarchyTree entries={unflatten($allResourcesData)}>
                  <div
                    let:itemName
                    use:contextMenu={{
                      name: "resource",
                      payload: { itemName },
                    }}
                    class="h-full w-full"
                    slot="itemName"
                  >
                    {itemName}
                  </div>
                </HierarchyTree>
              {/if}
              <HierarchyTree entries={fakeFileSystemEntries}>
                <div
                  let:itemName
                  use:contextMenu={{
                    name: "resource",
                    payload: { itemName },
                  }}
                  class="h-full w-full"
                  slot="itemName"
                >
                  {itemName}
                </div>
              </HierarchyTree>
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
  }

  .content-wrapper {
    @apply h-[calc(100vh-3.5rem)] w-full overflow-auto;
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
    @apply h-1/2;
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
    @apply h-1/2;
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
