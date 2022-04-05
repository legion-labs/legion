<script lang="ts">
  import { onMount } from "svelte";

  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import Tile from "@lgn/web-client/src/components/Tile.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";
  import { EmptyPanel, Panel } from "@lgn/web-client/src/components/panel";

  import DynamicPanel from "@/components/DynamicPanel.svelte";
  import ExtraPanel from "@/components/ExtraPanel.svelte";
  import ResourceBrowser from "@/components/ResourceBrowser.svelte";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import {
    allResourcesError,
    fetchAllResources,
  } from "@/orchestrators/allResources";
  import { currentResource } from "@/orchestrators/currentResource";
  import { currentResourceDescriptionEntry } from "@/orchestrators/resourceBrowserEntries";
  import contextMenu from "@/stores/contextMenu";
  import devSettings from "@/stores/devSettings";
  import modal from "@/stores/modal";
  import notifications from "@/stores/notifications";
  import { stagedResources, syncFromMain } from "@/stores/stagedResources";
  import workspace, { sceneExplorerTileId } from "@/stores/workspace";
  import { viewportTileId } from "@/stores/workspace";

  $: if ($allResourcesError) {
    refetchResources();
  }

  onMount(() => {
    refetchResources();
  });

  function refetchResources() {
    $currentResource = null;

    $currentResourceDescriptionEntry = null;

    return fetchAllResources();
  }
</script>

<ModalContainer store={modal} />

<ContextMenu store={contextMenu} />

<Notifications store={notifications} />

<div class="root">
  <TopBar devSettings={$devSettings} />
  <div class="content-wrapper" class:tauri={window.__TAURI_METADATA__}>
    <div class="content">
      <div class="secondary-contents">
        <div class="scene-explorer">
          <!-- TODO: Move this into a dedicated component DynamicTile -->
          <Tile id={sceneExplorerTileId} {workspace}>
            <div class="h-full w-full" slot="default" let:tile>
              {#if tile?.panel?.type === "populatedPanel"}
                <DynamicPanel panel={tile.panel} />
              {:else}
                <EmptyPanel>
                  <div class="empty-panel">
                    <em>No open scenes</em>
                  </div>
                </EmptyPanel>
              {/if}
            </div>
          </Tile>
        </div>
        <div class="h-separator" />
        <div class="resource-browser">
          <ResourceBrowser />
        </div>
      </div>
      <div class="v-separator" />
      <div class="main-content">
        <!-- TODO: Move this into a dedicated component DynamicTile -->
        <Tile id={viewportTileId} {workspace}>
          <div class="h-full w-full" slot="default" let:tile>
            {#if tile?.panel?.type === "populatedPanel"}
              <DynamicPanel panel={tile.panel} />
            {:else}
              <EmptyPanel>
                <div class="empty-panel">
                  <em>No open videos</em>
                </div>
              </EmptyPanel>
            {/if}
          </div>
        </Tile>
        <div class="h-separator" />
        <div class="extra-panel">
          <ExtraPanel />
        </div>
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
  <StatusBar {syncFromMain} stagedResources={$stagedResources || []} />
</div>

<style lang="postcss">
  .root {
    @apply h-screen w-full;
  }

  .root .content-wrapper {
    @apply h-[calc(100vh-4.5rem)] w-full overflow-auto;
  }

  .root .content-wrapper.tauri {
    @apply h-[calc(100vh-5rem)];
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

  .resource-browser {
    @apply h-[calc(50%-theme("spacing[0.5]"))];
  }

  .property-grid {
    @apply h-full;
  }

  .property-grid-content {
    @apply h-full;
  }

  .extra-panel {
    @apply h-80 flex-shrink-0;
  }

  .empty-panel {
    @apply flex items-center justify-center h-full w-full text-xl font-bold;
  }
</style>
