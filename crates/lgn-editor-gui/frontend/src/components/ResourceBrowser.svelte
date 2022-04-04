<script lang="ts">
  import Icon from "@iconify/svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import { UploadStatus } from "@lgn/proto-editor/dist/source_control";
  import { Panel, PanelHeader } from "@lgn/web-client/src/components/panel";
  import { readFile } from "@lgn/web-client/src/lib/files";
  import log from "@lgn/web-client/src/lib/log";
  import { createFilesStore } from "@lgn/web-client/src/stores/files";
  import { autoClose, select } from "@lgn/web-client/src/types/contextMenu";
  import type { Event as ContextMenuActionEvent } from "@lgn/web-client/src/types/contextMenu";

  import contextMenu from "@/actions/contextMenu";
  import {
    cloneResource,
    closeScene,
    createResource,
    getAllResources,
    initFileUpload,
    openScene,
    removeResource,
    renameResource,
    reparentResources,
    streamFileUpload,
  } from "@/api";
  import { resourceDragAndDropType } from "@/constants";
  import type { Entries, Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { components, join } from "@/lib/path";
  import { formatProperties } from "@/lib/propertyGrid";
  import type { ResourceProperty } from "@/lib/propertyGrid";
  import type { BagResourceProperty } from "@/lib/propertyGrid";
  import { iconFor } from "@/lib/resourceBrowser";
  import {
    currentResource,
    fetchCurrentResourceDescription,
  } from "@/orchestrators/currentResource";
  import allResources from "@/stores/allResources";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import modal from "@/stores/modal";
  import notifications from "@/stores/notifications";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";
  import CreateResourceModal from "./resources/CreateResourceModal.svelte";
  import ResourceFilter from "./resources/ResourceFilter.svelte";

  const createResourceModalId = Symbol.for("create-resource-modal");

  const files = createFilesStore();

  export let currentResourceDescriptionEntry: Entry<ResourceDescription> | null;

  export let resourceEntries: Entries<ResourceDescription>;

  export let currentlyRenameResourceEntry: Entry<ResourceDescription> | null;

  export let allResourcesLoading: boolean;

  let uploadingFiles = false;

  let resourceHierarchyTree: HierarchyTree<ResourceDescription> | null = null;

  let removePromptId: symbol | null = null;

  $: loading = uploadingFiles || allResourcesLoading;

  $: if ($files) {
    uploadFiles();
  }

  async function saveEditedResourceProperty({
    detail: { entry, newName },
  }: CustomEvent<{
    entry: Entry<ResourceDescription>;
    newName: string;
  }>) {
    const pathComponents = components(entry.item.path);

    if (!pathComponents) {
      return;
    }

    const newPath = join([...pathComponents.slice(0, -1), newName]);

    entry.item.path = newPath;

    try {
      await renameResource({ id: entry.item.id, newPath });

      allResources.run(getAllResources);
    } catch (error) {
      notifications.push(Symbol.for("resource-renaming-error"), {
        type: "error",
        title: "Resources",
        message: "An error occured while renaming the resource",
      });

      log.error(
        log.json`An error occured while renaming the resource ${entry.item}: ${error}`
      );
    }
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
          }).then(({ id, status }) => {
            if (!id || status === UploadStatus.REJECTED) {
              notifications.push(Symbol.for("file-upload"), {
                title: "File Upload",
                message: `File ${file.name} couldn't be uploaded`,
                type: "error",
              });

              return null;
            }

            return { id, name: file.name, file };
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
            new Promise<{ name: string; id: string }>((resolve, reject) => {
              streamFileUpload({
                id,
                content: new Uint8Array(content),
              }).subscribe({
                error(error) {
                  reject(error);
                },
                complete() {
                  resolve({ name, id });
                },
              });
            })
        );

        return [...acc, promise];
      }, [] as Promise<{ name: string; id: string }>[]);

      const names = await Promise.all(promises);

      let response = await Promise.all(
        names.map(({ name, id }) => {
          const lowerCasedName = name.toLowerCase().trim();

          if (lowerCasedName.endsWith(".png")) {
            return createResource({
              resourceName: name,
              resourceType: "png",
              parentResourceId: currentResourceDescriptionEntry?.item.id,
              uploadId: id,
            });
          }

          if (lowerCasedName.endsWith(".gltf.zip")) {
            // FIXME: Incorrect, should be an import
            return createResource({
              resourceName: name.slice(0, -4),
              resourceType: "gltf",
              parentResourceId: currentResourceDescriptionEntry?.item.id,
              uploadId: id,
            });
          }
        })
      );

      if (response && response[0]) {
        await allResources.run(getAllResources);
        let newId = response[0].newId;

        if (newId) {
          const entry = resourceEntries.find(
            (entry) => isEntry(entry) && entry.item.id == newId
          );

          if (!entry || !isEntry(entry)) {
            return;
          }

          currentResourceDescriptionEntry = entry;
          await fetchCurrentResourceDescription(newId);
        }
      }
    } catch (error) {
      log.error(log.json`File upload failed: ${error}`);
    } finally {
      uploadingFiles = false;
    }
  }

  function selectResource({
    detail: resourceDescription,
  }: CustomEvent<Entry<ResourceDescription>>) {
    if (resourceDescription) {
      fetchCurrentResourceDescription(resourceDescription.item.id);
    }
  }

  function filter({ detail: { name } }: CustomEvent<{ name: string }>) {
    allResources.run(() => getAllResources(name));
  }

  async function handleResourceActions({
    detail: { action, entrySetName },
  }: ContextMenuActionEvent<
    "resource" | "resourcePanel",
    Pick<ContextMenuEntryRecord, "resource" | "resourcePanel">
  >) {
    switch (action) {
      case "open_scene": {
        if (currentResourceDescriptionEntry) {
          await openScene({ id: currentResourceDescriptionEntry?.item.id });
        }

        return;
      }

      case "close_scene": {
        if (currentResourceDescriptionEntry) {
          await closeScene({ id: currentResourceDescriptionEntry?.item.id });
        }

        return;
      }

      case "clone": {
        if (!resourceHierarchyTree || !currentResourceDescriptionEntry) {
          return;
        }

        const { newResource } = await cloneResource({
          sourceId: currentResourceDescriptionEntry.item.id,
        });

        await allResources.run(getAllResources);

        if (newResource) {
          const entry = resourceEntries.find(
            (entry) => isEntry(entry) && entry.item.id == newResource.id
          );

          if (!entry || !isEntry(entry)) {
            return;
          }

          currentResourceDescriptionEntry = entry;

          fetchCurrentResourceDescription(newResource.id);
        }

        return;
      }

      case "import": {
        files.open({
          multiple: false,
          fileTypeSpecifiers: [".png", ".gltf.zip"],
        });

        return;
      }

      case "rename": {
        if (!resourceHierarchyTree || !currentResourceDescriptionEntry) {
          return;
        }

        currentlyRenameResourceEntry = currentResourceDescriptionEntry;

        return;
      }

      case "remove": {
        openRemoveResourcePrompt("request-resource-remove-context-menu");

        return;
      }

      case "new": {
        modal.open(createResourceModalId, CreateResourceModal, {
          payload: {
            resourceDescription:
              entrySetName === "resource"
                ? currentResourceDescriptionEntry?.item
                : null,
          },
        });

        return;
      }
    }
  }

  function refreshProperty(
    event: CustomEvent<{
      path: string;
      value: unknown;
    }>
  ) {
    if (!$currentResource) {
      log.error("No resources selected");

      return;
    }
    const resourceProperty = event.detail.value as ResourceProperty;

    if (resourceProperty) {
      for (const property of $currentResource.properties) {
        if (internalRefresh(event.detail.path, property, resourceProperty)) {
          break;
        }
      }
    }

    // Force refresh (TODO: try to only refresh what need to be refreshed)
    $currentResource.properties = $currentResource.properties;
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

  async function moveEntry({
    detail: { draggedEntry, dropzoneEntry },
  }: CustomEvent<{
    draggedEntry: Entry<ResourceDescription>;
    dropzoneEntry: Entry<ResourceDescription>;
  }>) {
    const newPath = dropzoneEntry.item.path;

    if (!newPath) {
      log.error(log.json`Couldn't find id for ${dropzoneEntry}`);

      return;
    }

    await reparentResources({
      id: draggedEntry.item.id,
      newPath,
    });

    await allResources.run(getAllResources);
  }

  function openRemoveResourcePrompt(symbolKey: string) {
    removePromptId = Symbol.for(symbolKey);

    modal.prompt(removePromptId);
  }

  async function removeResourceProperty({
    detail,
  }: CustomEvent<{ answer: boolean; id: symbol }>) {
    if (
      !removePromptId ||
      !resourceHierarchyTree ||
      !currentResourceDescriptionEntry ||
      !isEntry(currentResourceDescriptionEntry)
    ) {
      return;
    }

    const id = removePromptId;

    removePromptId = null;

    if (id !== detail.id || !detail.answer) {
      return;
    }

    const entry = resourceEntries.find(
      (entry) => entry === currentResourceDescriptionEntry
    );

    if (!entry) {
      return;
    }

    resourceEntries = resourceEntries.remove(entry);

    try {
      await removeResource({ id: currentResourceDescriptionEntry.item.id });
    } catch (error) {
      notifications.push(Symbol.for("resource-creation-error"), {
        type: "error",
        title: "Resources",
        message: "An error occured while removing the resource",
      });

      log.error(
        log.json`An error occured while removing the resource ${currentResourceDescriptionEntry.item}: ${error}`
      );
    }
  }
</script>

<svelte:window
  on:refresh-property={refreshProperty}
  on:contextmenu-action={autoClose(
    select(handleResourceActions, "resource", "resourcePanel")
  )}
  on:prompt-answer={removeResourceProperty}
/>

<Panel {loading} tabs={["Resource Browser"]}>
  <div slot="tab" let:tab>{tab}</div>

  <div slot="content" class="content" use:contextMenu={"resourcePanel"}>
    <PanelHeader>
      <ResourceFilter on:filter={filter} />
    </PanelHeader>
    <div class="hierarchy-tree">
      {#if !resourceEntries.isEmpty()}
        <HierarchyTree
          id="resource-browser"
          itemContextMenu="resource"
          renamable
          reorderable
          deletable
          draggable={resourceDragAndDropType}
          on:select={selectResource}
          on:nameEdited={saveEditedResourceProperty}
          on:moved={moveEntry}
          on:removeRequest={() =>
            openRemoveResourcePrompt("request-resource-remove-keyboard")}
          bind:entries={resourceEntries}
          bind:currentlyRenameEntry={currentlyRenameResourceEntry}
          bind:highlightedEntry={currentResourceDescriptionEntry}
          bind:this={resourceHierarchyTree}
        >
          <div class="w-full h-full" slot="icon" let:entry>
            <Icon class="w-full h-full" icon={iconFor(entry)} />
          </div>
          <div
            class="item"
            slot="name"
            let:entry
            title={isEntry(entry) ? entry.item.path : null}
          >
            {entry.name}
          </div>
        </HierarchyTree>
      {/if}
    </div>
  </div>
</Panel>

<style lang="postcss">
  .content {
    @apply h-full flex flex-col;
  }

  .hierarchy-tree {
    @apply flex-1 overflow-auto;
  }

  .item {
    @apply h-full w-full;
  }
</style>
