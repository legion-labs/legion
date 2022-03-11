<script lang="ts">
  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";
  import ResourceFilter from "./resources/ResourceFilter.svelte";
  import contextMenu from "@/actions/contextMenu";
  import {
    cloneResource,
    commitStagedResources,
    createResource,
    getAllResources,
    getStagedResources,
    initFileUpload,
    openScene,
    removeResource,
    renameResource,
    reparentResources,
    streamFileUpload,
    syncLatest,
  } from "@/api";
  import allResources from "@/stores/allResources";
  import { fetchCurrentResourceDescription } from "@/stores/currentResource";
  import { components, join } from "@/lib/path";
  import notifications from "@/stores/notifications";
  import type { Entries, Entry } from "@/lib/hierarchyTree";
  import log from "@lgn/web-client/src/lib/log";
  import { createFilesStore } from "@lgn/web-client/src/stores/files";
  import { UploadStatus } from "@lgn/proto-editor/dist/source_control";
  import { readFile } from "@lgn/web-client/src/lib/files";
  import { formatProperties } from "@/lib/propertyGrid";
  import type { ResourceProperty } from "@/lib/propertyGrid";
  import type { BagResourceProperty } from "@/lib/propertyGrid";
  import { autoClose, select } from "@lgn/web-client/src/types/contextMenu";
  import type { Event as ContextMenuActionEvent } from "@lgn/web-client/src/types/contextMenu";
  import currentResource from "@/stores/currentResource";
  import type { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import modal from "@/stores/modal";
  import CreateResourceModal from "./resources/CreateResourceModal.svelte";
  import Icon from "@iconify/svelte";
  import { iconFor } from "@/lib/resourceBrowser";

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

  const { data: currentResourceData } = currentResource;

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
            resourceName: name,
            resourceType: "png",
            parentResourceId: undefined,
          })
        )
      );

      allResources.run(getAllResources);
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
      fetchCurrentResourceDescription(resourceDescription.item);
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
            (entry) =>
              typeof entry.item !== "symbol" && entry.item.id == newResource.id
          );

          if (!entry || typeof entry.item === "symbol") {
            return;
          }

          currentResourceDescriptionEntry = entry;

          fetchCurrentResourceDescription(newResource);
        }

        return;
      }

      case "import": {
        files.open({ multiple: true, mimeTypes: ["image/png"] });

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

      case "sync_latest": {
        syncLatest();
        await allResources.run(getAllResources);

        return;
      }

      case "commit": {
        const result = await getStagedResources();

        log.error(result.entries);
        await commitStagedResources();

        return;
      }

      default: {
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
      typeof currentResourceDescriptionEntry.item === "symbol"
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
  <div slot="header">
    <ResourceFilter on:filter={filter} />
  </div>
  <div slot="content" class="content" use:contextMenu={"resourcePanel"}>
    {#if !resourceEntries.isEmpty()}
      <HierarchyTree
        withItemContextMenu="resource"
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
        <div class="item" slot="name" let:itemName>
          {itemName}
        </div>
      </HierarchyTree>
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .content {
    @apply h-full;
  }

  .item {
    @apply h-full w-full;
  }
</style>
