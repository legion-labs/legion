<script lang="ts">
  import Icon from "@iconify/svelte";
  import { createEventDispatcher } from "svelte";

  import {
    dropzone as dropzoneAction,
    isDragging,
    draggable as rawDraggableAction,
  } from "@lgn/web-client/src/actions/dnd";
  import { keyboardNavigationItem as keyboardNavigationItemAction } from "@lgn/web-client/src/actions/keyboardNavigation";
  import { nullable as nullableAction } from "@lgn/web-client/src/lib/action";

  import contextMenuAction from "@/actions/contextMenu";
  import { isEntry } from "@/lib/hierarchyTree";
  import type { Entry } from "@/lib/hierarchyTree";
  import { extension } from "@/lib/path";

  import TextInput from "../inputs/TextInput.svelte";

  type Item = $$Generic<{ id: string }>;

  type $$Slots = {
    name: { entry: Entry<Item | symbol> };
    icon: { entry: Entry<Item> };
  };

  const dispatch = createEventDispatcher<{
    highlight: Entry<Item>;
    nameEdited: { entry: Entry<Item>; newName: string };
    moved: { draggedEntry: Entry<Item>; dropzoneEntry: Entry<Item> };
  }>();

  export let id: string;

  export let index: number | null;

  export let entry: Entry<Item | symbol>;

  export let highlightedEntry: Entry<Item> | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let itemContextMenu: string | null = null;

  export let reorderable: boolean;

  export let draggable: string | null = null;

  /**
   * Currently highlighted entry _in the drag and drop context_
   * If a resource is dragged over an other resource this
   * variable will be populated by the entry that's being overed
   */
  export let dndHighlightedEntry: Entry<Item> | null;

  let mode: "view" | "edit";

  let isExpanded = true;

  /** Related to the inner drag and drop feature */
  $: moveInnerEntryType = `hierarchy-tree-entry-${id}`;

  // TODO: Use a filter instead
  $: isDisabled = !isEntry(entry);

  $: isHighlighted = highlightedEntry ? entry === highlightedEntry : false;

  $: mode =
    currentlyRenameEntry && currentlyRenameEntry === entry ? "edit" : "view";

  $: nameValue = mode === "edit" ? entryName() : "";

  $: if (!$isDragging) {
    dndHighlightedEntry = null;
  }

  $: if (!isHighlighted) {
    cancelNameEdit();
  }

  const draggableAction = nullableAction(rawDraggableAction);

  const dropzone = nullableAction(dropzoneAction);

  const contextMenu = nullableAction(contextMenuAction);

  const keyboardNavigationItem = nullableAction(keyboardNavigationItemAction);

  function extractAutoSelectRange() {
    const name = entryName();

    const ext = extension(name);

    if (ext == null) {
      return true;
    }

    // -1 for the '.'
    return [0, name.length - ext.length - 1] as const;
  }

  function highlight() {
    if (isDisabled || !isEntry(entry)) {
      return;
    }

    dispatch("highlight", entry);
  }

  function renameEntry() {
    if (isDisabled || !isEntry(entry)) {
      return;
    }

    currentlyRenameEntry = null;

    if (nameValue.trim().length) {
      dispatch("nameEdited", { entry, newName: nameValue.trim() });
    }
  }

  function cancelNameEdit() {
    mode = "view";
  }

  function entryName() {
    return entry.name.trim();
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  function onDragEnter({
    detail: { originalEvent },
  }: CustomEvent<{ originalEvent: DragEvent }>) {
    originalEvent.stopPropagation();

    if (isDisabled || !isEntry(entry)) {
      return;
    }

    dndHighlightedEntry = entry;
  }

  function onDrop({
    detail: { item: draggedEntry },
  }: CustomEvent<{ item: Entry<Item> }>) {
    if (isDisabled || !isEntry(entry)) {
      return;
    }

    dispatch("moved", {
      draggedEntry,
      dropzoneEntry: entry,
    });
  }

  function onFormKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      cancelNameEdit();
    }
  }
</script>

<div
  class="root"
  class:bg-gray-800={dndHighlightedEntry === entry}
  on:dblclick
  use:keyboardNavigationItem={isDisabled ? null : index}
  use:dropzone={!isDisabled && reorderable
    ? { accept: moveInnerEntryType }
    : null}
  on:dnd-drop={onDrop}
  on:dnd-dragenter={onDragEnter}
>
  <div
    class="name"
    class:disabled={isDisabled}
    class:font-semibold={entry.subEntries.length}
    class:lg-space={mode === "view"}
    class:highlighted-view={isHighlighted && mode === "view"}
    on:mousedown={highlight}
    use:contextMenu={isDisabled ? null : itemContextMenu}
    use:draggableAction={!isDisabled && reorderable
      ? { item: entry, type: moveInnerEntryType }
      : null}
    use:draggableAction={!isDisabled && draggable
      ? { item: entry, type: draggable }
      : null}
  >
    {#if entry.subEntries.length > 0}
      <div class="icon-container" on:click={toggleExpanded}>
        <div class="folder-icon" class:expanded={isExpanded}>
          <Icon icon="ic:baseline-chevron-right" />
        </div>
        <div class="icon">
          {#if !isEntry(entry)}
            <Icon class="w-full h-full" icon="ic:baseline-folder-open" />
          {:else}
            <slot name="icon" {entry} />
          {/if}
        </div>
      </div>
    {:else}
      <div class="icon">
        {#if !isEntry(entry)}
          <Icon class="w-full h-full" icon="ic:baseline-folder-open" />
        {:else}
          <slot name="icon" {entry} />
        {/if}
      </div>
    {/if}
    <div class="name">
      {#if mode === "view"}
        <slot name="name" {entry} />
      {:else}
        <form
          on:submit|stopPropagation|preventDefault={renameEntry}
          on:keydown|stopPropagation={onFormKeydown}
        >
          <TextInput
            autoFocus
            autoSelect={extractAutoSelectRange()}
            size="sm"
            bind:value={nameValue}
          />
        </form>
      {/if}
    </div>
  </div>
  {#if entry.subEntries.length && isExpanded}
    {#each entry.subEntries as subEntry (isEntry(subEntry) ? subEntry.item.id : subEntry.item)}
      <div class="sub-entries">
        <svelte:self
          {id}
          {highlightedEntry}
          {itemContextMenu}
          {reorderable}
          {draggable}
          index={subEntry.index}
          entry={subEntry}
          bind:currentlyRenameEntry
          bind:dndHighlightedEntry
          on:highlight
          on:nameEdited
          on:moved
          let:entry
        >
          <slot name="icon" slot="icon" {entry} />
          <slot name="name" slot="name" {entry} />
        </svelte:self>
      </div>
    {/each}
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col cursor-pointer border-dotted border-gray-400;
  }

  .name {
    @apply flex items-center h-7 w-full px-1 border border-transparent;
  }

  .name.disabled {
    @apply cursor-not-allowed;
  }

  .name.highlighted-view {
    @apply border border-dotted border-orange-700 bg-orange-700 bg-opacity-10;
  }

  .name.lg-space {
    @apply space-x-1;
  }

  .icon-container {
    @apply flex items-center space-x-1;
  }

  .icon {
    @apply flex items-center h-5 w-5 text-orange-700 opacity-50;
  }

  .folder-icon {
    @apply flex items-center h-2.5 w-2.5 transition-all duration-150 cursor-pointer;
  }

  .folder-icon.expanded {
    @apply rotate-90;
  }

  .sub-entries {
    @apply pl-2 ml-3 list-none border-l border-dotted border-gray-400;
  }
</style>
