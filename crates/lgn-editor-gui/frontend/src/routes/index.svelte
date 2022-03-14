<script lang="ts">
  import { Panel } from "@lgn/web-client/src/components/panel";
  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import ViewportPanel from "@lgn/web-client/src/components/panel/ViewportPanel.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import { getAllResources, streamLogs } from "@/api";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import currentResource from "@/orchestrators/currentResource";
  import { createHierarchyTreeOrchestrator } from "@/orchestrators/hierarchyTree";
  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import contextMenu from "@/stores/contextMenu";
  import allResourcesStore from "@/stores/allResources";
  import viewportOrchestrator from "@/orchestrators/viewport";
  import { onMount } from "svelte";
  import authStatus from "@/stores/authStatus";
  import AuthModal from "@/components/AuthModal.svelte";
  import notifications from "@/stores/notifications";
  import SceneExplorer from "@/components/SceneExplorer.svelte";
  import ResourceBrowser from "@/components/ResourceBrowser.svelte";
  import modal from "@/stores/modal";
  import type { Entry } from "@/lib/hierarchyTree";
  import { tap } from "rxjs/operators";
  import log from "@lgn/web-client/src/lib/log";

  const { data: currentResourceData } = currentResource;

  const {
    data: allResourcesData,
    error: allResourcesError,
    loading: allResourcesLoading,
  } = allResourcesStore;

  const resourceEntriesOrchestrator =
    createHierarchyTreeOrchestrator<ResourceDescription>();

  const {
    currentlyRenameEntry: currentlyRenameResourceEntry,
    entries: resourceEntries,
  } = resourceEntriesOrchestrator;

  let currentResourceDescriptionEntry: Entry<ResourceDescription> | null = null;

  $: currentResourceDescription = currentResourceDescriptionEntry?.item ?? null;

  $: if ($allResourcesError) {
    reloadResources();
  }

  $: if ($allResourcesData) {
    resourceEntriesOrchestrator.load($allResourcesData);
  }

  onMount(() => {
    reloadResources();

    streamLogs().then((logs) => {
      logs
        .pipe(
          tap(({ time, target, level, message }) =>
            log.trace(`${time} - ${target} - ${message}`)
          )
        )
        .subscribe();
    });

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

    currentResourceDescriptionEntry = entry;
  }

  async function reloadResources() {
    $currentResourceData = null;
    currentResourceDescriptionEntry = null;
    await allResourcesStore.run(getAllResources);
  }
</script>

<ModalContainer store={modal} />

<ContextMenu store={contextMenu} />

<Notifications store={notifications} />

<div class="root">
  <TopBar />
  <div class="content-wrapper">
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
            bind:currentResourceDescriptionEntry
            bind:currentlyRenameResourceEntry={$currentlyRenameResourceEntry}
            bind:resourceEntries={$resourceEntries}
          />
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
  }

  .root .content-wrapper {
    @apply h-[calc(100vh-4rem)] w-full overflow-auto;
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
</style>
