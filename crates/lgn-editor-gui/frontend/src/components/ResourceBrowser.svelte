<script lang="ts">
  import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import HierarchyTree from "./hierarchyTree/HierarchyTree.svelte";
  import ResourceFilter from "./resources/ResourceFilter.svelte";
  import contextMenu from "@/actions/contextMenu";
  import {
    cloneResource,
    createResource,
    getAllResources,
    initFileUpload,
    removeResource,
    renameResource,
    streamFileUpload,
  } from "@/api";
  import allResources from "@/stores/allResources";
  import { fetchCurrentResourceDescription } from "@/stores/currentResource";
  import { components, join } from "@/lib/path";
  import notifications from "@/stores/notifications";
  import { Entries, Entry } from "@/lib/hierarchyTree";
  import log from "@lgn/web-client/src/lib/log";
  import Files from "@lgn/web-client/src/stores/files";
  import { UploadStatus } from "@lgn/proto-editor/dist/source_control";
  import { readFile } from "@lgn/web-client/src/lib/files";
  import {
    BagResourceProperty,
    formatProperties,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import {
    autoClose,
    Event as ContextMenuActionEvent,
    select,
  } from "@lgn/web-client/src/types/contextMenu";
  import currentResource from "@/stores/currentResource";
  import { ContextMenuEntryRecord } from "@/stores/contextMenu";
  import modal from "@/stores/modal";
  import CreateResourceModal from "./resources/CreateResourceModal.svelte";

  const createResourceModalId = Symbol();

  const files = new Files();

  export let currentResourceDescription: ResourceDescription | null;

  export let resourceEntries: Entries<symbol | ResourceDescription>;

  export let currentlyRenameResource: Entry<
    symbol | ResourceDescription
  > | null;

  export let allResourcesLoading: boolean;

  let uploadingFiles = false;

  let resourceHierarchyTree: HierarchyTree<
    ResourceDescription | symbol
  > | null = null;

  $: loading = uploadingFiles || allResourcesLoading;

  $: if ($files) {
    uploadFiles();
  }

  const { data: currentResourceData } = currentResource;

  async function removeResourceProperty({
    detail: entry,
  }: CustomEvent<Entry<ResourceDescription | symbol>>) {
    if (typeof entry.item === "symbol") {
      return;
    }

    try {
      await removeResource({ id: entry.item.id });
    } catch (error) {
      notifications.push(Symbol(), {
        type: "error",
        title: "Resources",
        message: "An error occured while removing the resource",
      });

      log.error(
        log.json`An error occured while removing the resource ${entry.item}: ${error}`
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

    entry.item.path = newPath;

    try {
      await renameResource({ id: entry.item.id, newPath });
    } catch (error) {
      notifications.push(Symbol(), {
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
            resourcePath: name,
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
  }: CustomEvent<Entry<ResourceDescription | symbol>>) {
    resourceDescription &&
      !(typeof resourceDescription.item === "symbol") &&
      fetchCurrentResourceDescription(resourceDescription.item);
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
      case "clone": {
        if (!resourceHierarchyTree || !currentResourceDescription) {
          return;
        }

        const detail = await cloneResource({
          sourceId: currentResourceDescription.id,
        });

        await allResources.run(getAllResources);

        if (detail.newResource) {
          resourceHierarchyTree.forceSelection(detail.newResource.id);
          fetchCurrentResourceDescription(detail.newResource);
        }

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
        modal.open(createResourceModalId, CreateResourceModal, {
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
</script>

<svelte:window
  on:refresh-property={refreshProperty}
  on:contextmenu-action={autoClose(
    select(handleResourceActions, "resource", "resourcePanel")
  )}
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
        on:removed={removeResourceProperty}
        bind:entries={resourceEntries}
        bind:currentlyRenameEntry={currentlyRenameResource}
        bind:highlightedItem={currentResourceDescription}
        bind:this={resourceHierarchyTree}
      >
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
