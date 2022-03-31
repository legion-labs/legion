<script lang="ts">
  import { onMount } from "svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";
  import { DynamicPanel, Panel } from "@lgn/web-client/src/components/panel";
  import log from "@lgn/web-client/src/lib/log";

  import { getActiveScenes, getAllResources } from "@/api";
  import AuthModal from "@/components/AuthModal.svelte";
  import ExtraPanel from "@/components/ExtraPanel.svelte";
  import ResourceBrowser from "@/components/ResourceBrowser.svelte";
  import SceneExplorer from "@/components/SceneExplorer.svelte";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import { currentResource } from "@/orchestrators/currentResource";
  import {
    currentResourceDescriptionEntry,
    currentlyRenameResourceEntry,
    resourceBrowserEntriesOrchestrator,
    resourceEntries,
  } from "@/orchestrators/resourceBrowserEntries";
  import allResourcesStore from "@/stores/allResources";
  import authStatus from "@/stores/authStatus";
  import contextMenu from "@/stores/contextMenu";
  import modal from "@/stores/modal";
  import notifications from "@/stores/notifications";
  import { stagedResources, syncFromMain } from "@/stores/stagedResources";
  import workspace, { viewportPanelKey } from "@/stores/workspace";

  const {
    data: allResourcesData,
    error: allResourcesError,
    loading: allResourcesLoading,
  } = allResourcesStore;

  $: currentResourceDescription =
    $currentResourceDescriptionEntry?.item ?? null;

  $: if ($allResourcesError) {
    reloadResources();
  }

  $: if ($allResourcesData) {
    resourceBrowserEntriesOrchestrator.load($allResourcesData);
  }

  onMount(() => {
    reloadResources();

    if ($authStatus && $authStatus.type === "error") {
      modal.open(Symbol.for("auth-modal"), AuthModal, {
        payload: { authorizationUrl: $authStatus.authorizationUrl },
        noTransition: true,
      });
    }
  });

  function setCurrentDescriptionEntry({
    detail: resource,
  }: CustomEvent<ResourceDescription>) {
    const entry = $resourceEntries.find((entry) => entry.item === resource);

    if (!entry) {
      return;
    }

    $currentResourceDescriptionEntry = entry;
  }

  async function reloadResources() {
    $currentResource = null;

    $currentResourceDescriptionEntry = null;

    await allResourcesStore.run(getAllResources);

    let active_scenes = await getActiveScenes();
    log.info("Active Scenes: ", active_scenes.sceneIds);
  }
</script>

<ModalContainer store={modal} />

<ContextMenu store={contextMenu} />

<Notifications store={notifications} />

<div class="root">
  <TopBar />
  <div class="content-wrapper" class:tauri={window.__TAURI_METADATA__}>
    <div class="content">
      <div class="secondary-contents">
        <div class="scene-explorer">
          <SceneExplorer
            allResourcesLoading={$allResourcesLoading}
            resourceEntries={$resourceEntries}
            {currentResourceDescription}
            on:currentResourceDescriptionChange={setCurrentDescriptionEntry}
          />
        </div>
        <div class="h-separator" />
        <div class="resource-browser">
          <ResourceBrowser
            allResourcesLoading={$allResourcesLoading}
            bind:currentResourceDescriptionEntry={$currentResourceDescriptionEntry}
            bind:currentlyRenameResourceEntry={$currentlyRenameResourceEntry}
            bind:resourceEntries={$resourceEntries}
          />
        </div>
      </div>
      <div class="v-separator" />
      <div class="main-content">
        <DynamicPanel panelKey={viewportPanelKey} {workspace} />
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
  <StatusBar stagedResources={$stagedResources || []} {syncFromMain} />
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
</style>
