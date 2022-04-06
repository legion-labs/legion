<script lang="ts">
  import Icon from "@iconify/svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import { UploadStatus } from "@lgn/proto-editor/dist/source_control";
  import HighlightedText from "@lgn/web-client/src/components/HighlightedText.svelte";
  import { Panel, PanelHeader } from "@lgn/web-client/src/components/panel";
  import { displayError } from "@lgn/web-client/src/lib/errors";
  import { readFile } from "@lgn/web-client/src/lib/files";
  import log from "@lgn/web-client/src/lib/log";
  import { createFilesStore } from "@lgn/web-client/src/stores/files";
  import { filterContextMenuEvents } from "@lgn/web-client/src/types/contextMenu";
  import type { ContextMenuEvent } from "@lgn/web-client/src/types/contextMenu";

  import contextMenu from "@/actions/contextMenu";
  import {
    cloneResource,
    createResource,
    initFileUpload,
    openScene,
    removeResource,
    renameResource,
    reparentResources,
    streamFileUpload,
  } from "@/api";
  import { resourceDragAndDropType } from "@/constants";
  import { Entries } from "@/lib/hierarchyTree";
  import type { Entry } from "@/lib/hierarchyTree";
  import { isEntry } from "@/lib/hierarchyTree";
  import { components, join } from "@/lib/path";
  import { formatProperties } from "@/lib/propertyGrid";
  import type { ResourceProperty } from "@/lib/propertyGrid";
  import type { BagResourceProperty } from "@/lib/propertyGrid";
  import { iconFor } from "@/lib/resourceBrowser";
  import { fetchAllActiveScenes } from "@/orchestrators/allActiveScenes";
  import {
    allResources,
    allResourcesLoading,
    fetchAllResources,
  } from "@/orchestrators/allResources";
  import {
    currentResource,
    fetchCurrentResourceDescription,
  } from "@/orchestrators/currentResource";
  import {
    currentResourceDescriptionEntry,
    currentlyRenameResourceEntry,
    resourceEntries,
    resourceEntriesfilters,
  } from "@/orchestrators/resourceBrowserEntries";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import {
    resourceBrowserItemContextMenuId,
    resourceBrowserPanelContextMenuId,
  } from "@/stores/contextMenu";
  import modal from "@/stores/modal";
  import notifications from "@/stores/notifications";

  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";
  import CreateResourceModal from "./resources/CreateResourceModal.svelte";
  import ResourceFilter from "./resources/ResourceFilter.svelte";

  const createResourceModalId = Symbol.for("create-resource-modal");

  const files = createFilesStore();

  let uploadingFiles = false;

  let resourceHierarchyTree: HierarchyTree<ResourceDescription> | null = null;

  let removePromptId: symbol | null = null;

  $: loading = uploadingFiles || $allResourcesLoading;

  $: if ($files) {
    uploadFiles();
  }

  $: filteredResourceEntries =
    $allResources && $resourceEntriesfilters.name
      ? Entries.fromArray(
          $allResources.filter((resource) =>
            $resourceEntriesfilters.name
              ? resource.path
                  .toLowerCase()
                  .includes($resourceEntriesfilters.name.toLowerCase())
              : true
          )
        )
      : null;

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

      await fetchAllResources();
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

      let [newResource] = await Promise.all(
        names.map(({ name, id }) => {
          const lowerCasedName = name.toLowerCase().trim();

          if (lowerCasedName.endsWith(".png")) {
            return createResource({
              resourceName: name,
              resourceType: "png",
              parentResourceId: $currentResourceDescriptionEntry?.item.id,
              uploadId: id,
            });
          }

          if (lowerCasedName.endsWith(".gltf.zip")) {
            // FIXME: Incorrect, should be an import
            return createResource({
              resourceName: name.slice(0, -4),
              resourceType: "gltf",
              parentResourceId: $currentResourceDescriptionEntry?.item.id,
              uploadId: id,
            });
          }
        })
      );

      if (newResource) {
        await fetchAllResources();

        const newId = newResource.newId;

        if (newId) {
          const entry = $resourceEntries.find(
            (entry) => isEntry(entry) && entry.item.id == newId
          );

          if (!entry || !isEntry(entry)) {
            return;
          }

          $currentResourceDescriptionEntry = entry;

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
    $resourceEntriesfilters.name = name;
  }

  async function handleContextMenuEvents({
    detail: { action, close, entrySetName },
  }: ContextMenuEvent<
    | typeof resourceBrowserItemContextMenuId
    | typeof resourceBrowserPanelContextMenuId,
    Pick<
      ContextMenuEntryRecord,
      | typeof resourceBrowserItemContextMenuId
      | typeof resourceBrowserPanelContextMenuId
    >
  >) {
    close();

    switch (action) {
      case "openScene": {
        if ($currentResourceDescriptionEntry) {
          try {
            await openScene({ id: $currentResourceDescriptionEntry?.item.id });

            await fetchAllActiveScenes();
          } catch (error) {
            notifications.push(Symbol(), {
              title: "Scene Explorer",
              message: displayError(error),
              type: "error",
            });
          }
        }

        break;
      }

      case "clone": {
        if (!resourceHierarchyTree || !$currentResourceDescriptionEntry) {
          break;
        }

        const { newResource } = await cloneResource({
          sourceId: $currentResourceDescriptionEntry.item.id,
        });

        await fetchAllResources();

        if (newResource) {
          const entry = $resourceEntries.find(
            (entry) => isEntry(entry) && entry.item.id == newResource.id
          );

          if (!entry || !isEntry(entry)) {
            break;
          }

          $currentResourceDescriptionEntry = entry;

          fetchCurrentResourceDescription(newResource.id);
        }

        break;
      }

      case "import": {
        files.open({
          multiple: false,
          fileTypeSpecifiers: [".png", ".gltf.zip"],
        });

        break;
      }

      case "rename": {
        if (!resourceHierarchyTree || !$currentResourceDescriptionEntry) {
          break;
        }

        $currentlyRenameResourceEntry = $currentResourceDescriptionEntry;

        break;
      }

      case "remove": {
        openRemoveResourcePrompt("request-resource-remove-context-menu");

        break;
      }

      case "new": {
        modal.open(createResourceModalId, CreateResourceModal, {
          payload: {
            resourceDescription:
              entrySetName === "resourceBrowserItemContextMenu"
                ? $currentResourceDescriptionEntry?.item
                : null,
          },
        });

        break;
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

    await fetchAllResources();
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
      !$currentResourceDescriptionEntry ||
      !isEntry($currentResourceDescriptionEntry)
    ) {
      return;
    }

    const id = removePromptId;

    removePromptId = null;

    if (id !== detail.id || !detail.answer) {
      return;
    }

    const entry = $resourceEntries.find(
      (entry) => entry.item.id === $currentResourceDescriptionEntry?.item.id
    );

    if (!entry) {
      return;
    }

    $resourceEntries = $resourceEntries.remove(entry);

    try {
      await removeResource({ id: $currentResourceDescriptionEntry.item.id });
    } catch (error) {
      notifications.push(Symbol.for("resource-creation-error"), {
        type: "error",
        title: "Resources",
        message: "An error occured while removing the resource",
      });

      log.error(
        log.json`An error occured while removing the resource ${$currentResourceDescriptionEntry.item}: ${error}`
      );
    }
  }
</script>

<svelte:window
  on:refresh-property={refreshProperty}
  on:contextmenu-action={filterContextMenuEvents(
    handleContextMenuEvents,
    resourceBrowserItemContextMenuId,
    resourceBrowserPanelContextMenuId
  )}
  on:prompt-answer={removeResourceProperty}
/>

<Panel {loading} tabs={["Resource Browser"]}>
  <div slot="tab" let:tab>{tab}</div>

  <div
    slot="content"
    class="content"
    use:contextMenu={resourceBrowserPanelContextMenuId}
  >
    <PanelHeader>
      <ResourceFilter on:filter={filter} />
    </PanelHeader>
    <div class="hierarchy-tree">
      {#if !$resourceEntries.isEmpty()}
        <HierarchyTree
          id="resource-browser"
          renamable
          reorderable
          deletable
          draggable={resourceDragAndDropType}
          displayedEntries={filteredResourceEntries || $resourceEntries}
          on:select={selectResource}
          on:nameEdited={saveEditedResourceProperty}
          on:moved={moveEntry}
          on:removeRequest={() =>
            openRemoveResourcePrompt("request-resource-remove-keyboard")}
          bind:entries={$resourceEntries}
          bind:currentlyRenameEntry={$currentlyRenameResourceEntry}
          bind:highlightedEntry={$currentResourceDescriptionEntry}
          bind:this={resourceHierarchyTree}
        >
          <div
            class="entry"
            slot="entry"
            use:contextMenu={resourceBrowserItemContextMenuId}
            let:entry
            let:isHighlighted
          >
            <div
              class="entry-icon"
              class:text-gray-400={!isHighlighted}
              class:text-orange-700={isHighlighted}
            >
              <Icon class="w-full h-full" icon={iconFor(entry)} />
            </div>
            <div
              class="entry-name"
              title={isEntry(entry) ? entry.item.path : null}
            >
              {#if $resourceEntriesfilters.name}
                <HighlightedText
                  text={entry.name}
                  pattern={$resourceEntriesfilters.name}
                />
              {:else}
                {entry.name}
              {/if}
            </div>
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

  .entry {
    @apply flex flex-row w-full h-full space-x-1;
  }

  .entry-icon {
    @apply w-6 h-6;
  }

  .entry-name {
    @apply w-full h-full;
  }
</style>
