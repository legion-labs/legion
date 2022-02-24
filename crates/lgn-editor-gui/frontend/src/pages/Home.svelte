<script lang="ts">
  import { bufferCount, map, tap, mergeMap } from "rxjs/operators";
  import { Panel, PanelList } from "@lgn/web-client/src/components/panel";
  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import ViewportPanel from "@lgn/web-client/src/components/panel/ViewportPanel.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import {
    cloneResource,
    createResource,
    getAllResources,
    getResourceProperties,
    initFileUpload,
    removeResource,
    renameResource,
    streamFileUpload,
    updateSelection,
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
  import allResourcesStore from "@/stores/allResources";
  import viewportOrchestrator from "@/stores/viewport";
  import notificationsStore from "@/stores/notifications";
  import { components, join } from "@/lib/path";
  import ResourceFilter from "@/components/resources/ResourceFilter.svelte";
  import { onMount } from "svelte";
  import authStatus from "@/stores/authStatus";
  import Files from "@lgn/web-client/src/stores/files";
  import AuthModal from "@/components/AuthModal.svelte";
  import {
    BagResourceProperty,
    formatProperties,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import { UploadStatus } from "@lgn/proto-editor/dist/source_control";
  import { readFile } from "@lgn/web-client/src/lib/files";
  import notifications from "@/stores/notifications";

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

  const {
    data: allResourcesData,
    error: allResourcesError,
    loading: allResourcesLoading,
  } = allResourcesStore;

  const files = new Files();

  let uploadingFiles = false;

  let currentResourceDescription: ResourceDescription | null = null;

  let resourceHierarchyTree: HierarchyTree<
    ResourceDescription | symbol
  > | null = null;

  $: if ($allResourcesData) {
    resourceEntriesOrchestrator.load($allResourcesData);
  }

  $: if ($files) {
    uploadFiles();
  }

  allResourcesStore.run(getAllResources);

  onMount(() => {
    if ($authStatus && $authStatus.type === "error") {
      modalStore.open(Symbol.for("auth-modal"), AuthModal, {
        payload: $authStatus.authorizationUrl,
        noTransition: true,
      });
    }
  });

  function fetchCurrentResourceDescription() {
    if (!currentResourceDescription) {
      return;
    }

    try {
      currentResourceStore.run(() => {
        if (!currentResourceDescription) {
          throw new Error("Current resource description not found");
        }

        updateSelection(currentResourceDescription.id);

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

  function refreshProperty(
    event: CustomEvent<{
      path: string;
      value: unknown;
    }>
  ) {
    if (!$currentResourceData) {
      log.error("No resources selected");

      return;
    }
    const resourceProperty = event.detail.value as ResourceProperty;

    if (resourceProperty) {
      for (const property of $currentResourceData.properties) {
        if (internalRefresh(event.detail.path, property, resourceProperty)) {
          break;
        }
      }
    }

    // Force refresh (TODO: try to only refresh what need to be refreshed)
    $currentResourceData.properties = $currentResourceData.properties;
  }

  function internalRefresh(
    restOfPath: string,
    base: ResourceProperty,
    value: ResourceProperty
  ): boolean {
    if (base as BagResourceProperty) {
      if (restOfPath == "") {
        const formatted = formatProperties([value])[0];

        let found = base.subProperties.find((v) => v.name == value.name);

        if (found) {
          found = formatted;
        } else {
          base.subProperties.push(formatted);
        }

        return true;
      }

      for (const property of base.subProperties) {
        if (restOfPath.startsWith(property.name)) {
          restOfPath = restOfPath.substring(property.name.length);

          if (restOfPath.startsWith(".")) {
            restOfPath = restOfPath.slice(1);
          }

          return internalRefresh(restOfPath, property, value);
        }
      }
    }

    return false;
  }

  async function tryAgain() {
    $currentResourceData = null;
    currentResourceDescription = null;
    await allResourcesStore.run(getAllResources);
  }

  async function uploadFiles() {
    if (!$files || !$files.length) {
      return;
    }

    uploadingFiles = true;

    try {
      const filesWithId = await Promise.all(
        $files.map((file) =>
          initFileUpload({
            name: file.name,
            size: file.size,
          }).then(({ id, name, status }) => {
            if (!id || !name || status === UploadStatus.REJECTED) {
              notifications.push(Symbol.for("file-upload"), {
                title: "File Upload",
                message: `File ${file.name} couldn't be uploaded`,
                type: "error",
              });

              return null;
            }

            return { id, name, file };
          })
        )
      );

      const promises = filesWithId.reduce((acc, fileWithId) => {
        if (!fileWithId) {
          return acc;
        }

        const { id, name, file } = fileWithId;

        const promise = readFile(file).then(
          (content) =>
            new Promise<string>((resolve, reject) => {
              streamFileUpload({
                id,
                content: new Uint8Array(content),
              }).subscribe({
                error(error) {
                  reject(error);
                },
                next({ progress }) {
                  console.log("ID", progress?.id);
                },
                complete() {
                  resolve(name);
                },
              });
            })
        );

        return [...acc, promise];
      }, [] as Promise<string>[]);

      const names = await Promise.all(promises);

      await Promise.all(
        names.map((name) =>
          createResource({
            resourcePath: name,
            resourceType: "png",
          })
            .then(console.log.bind(console, "Ok: "))
            .catch(console.log.bind(console, "Error: "))
        )
      );

      allResourcesStore.run(getAllResources);
    } catch (error) {
      log.error(log.json`File upload failed: ${error}`);
    } finally {
      uploadingFiles = false;
    }
  }

  async function handleResourceRename({
    detail: { action, entrySetName },
  }: ContextMenuActionEvent<
    "resource" | "resourcePanel",
    Pick<ContextMenuEntryRecord, "resource" | "resourcePanel">
  >) {
    switch (action) {
      case "clone": {
        if (!resourceHierarchyTree || !currentResourceDescription) {
          return;
        }

        await cloneResource({ sourceId: currentResourceDescription.id });

        await allResourcesStore.run(getAllResources);

        return;
      }

      case "import": {
        files.open({ multiple: true, mimeTypes: ["image/png"] });

        return;
      }

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
        modalStore.open(createResourceModalId, CreateResourceModal, {
          payload:
            entrySetName === "resource" ? currentResourceDescription : null,
        });

        return;
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
  on:refresh-property={refreshProperty}
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
          <Panel tabs={["Scene Explorer"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div slot="content" class="scene-explorer-content" let:isFocused>
              {#if $allResourcesData}
                <PanelList
                  key="id"
                  items={$allResourcesData || []}
                  panelIsFocused={isFocused}
                  on:select={fetchCurrentResourceDescription}
                  bind:highlightedItem={currentResourceDescription}
                >
                  <div slot="default" let:item={resource}>
                    {resource.path}
                  </div>
                </PanelList>
              {:else if $allResourcesLoading}
                <div class="scene-explorer-loading">Loading...</div>
              {:else if $allResourcesError}
                <div class="scene-explorer-error">
                  An error occured while fetching the scene explorer
                  <span class="scene-explorer-try-again" on:click={tryAgain}>
                    try again
                  </span>
                </div>
              {/if}
            </div>
          </Panel>
        </div>
        <div class="h-separator" />
        <div class="resource-browser">
          <Panel loading={uploadingFiles} tabs={["Resource Browser"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div slot="header">
              <ResourceFilter
                on:filter={({ detail: { name } }) =>
                  allResourcesStore.run(() => getAllResources(name))}
              />
            </div>
            <div
              slot="content"
              class="resource-browser-content"
              use:contextMenu={"resourcePanel"}
            >
              {#if $allResourcesData}
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
              {:else if $allResourcesLoading}
                <div class="scene-explorer-loading">Loading...</div>
              {:else if $allResourcesError}
                <div class="scene-explorer-error">
                  An error occured while fetching the scene explorer
                  <span class="scene-explorer-try-again" on:click={tryAgain}>
                    try again
                  </span>
                </div>
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
              <PropertyGrid {modalStore} />
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
