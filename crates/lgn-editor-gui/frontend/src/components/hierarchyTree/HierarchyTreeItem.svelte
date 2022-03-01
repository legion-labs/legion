<script lang="ts">
  import { Entry } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";
  import { extension } from "@/lib/path";
  import Icon from "@iconify/svelte";
  import { keyboardNavigationItem } from "@lgn/web-client/src/actions/keyboardNavigation";
  import contextMenuAction from "@/actions/contextMenu";
  import TextInput from "../inputs/TextInput.svelte";
  import {
    isDragging,
    dropzone as dropzoneAction,
    draggable as draggableAction,
  } from "@lgn/web-client/src/actions/dnd";
  import { nullable as nullableAction } from "@lgn/web-client/src/lib/action";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
    icon: { entry: Entry<Item> };
  };

  const dispatch = createEventDispatcher<{
    highlight: Entry<Item>;
    nameEdited: { entry: Entry<Item>; newName: string };
    moved: { draggedEntry: Entry<Item>; dropzoneEntry: Entry<Item> };
  }>();

  // DnD type
  // TODO: Will probably have to be shared throughout the whole application
  const type = "RESOURCE";

  export let index: number;

  export let entry: Entry<Item>;

  export let highlightedEntry: Entry<Item> | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let withItemContextMenu: string | null = null;

  /**
   * Currently highlighted entry _in the drag and drop context_
   * If a resource is dragged over an other resource this
   * variable will be populated by the entry that's being overed
   */
  export let dndHighlightedEntry: Entry<Item> | null;

  let mode: "view" | "edit";

  let isExpanded = true;

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

  const draggable = nullableAction(draggableAction);

  const dropzone = nullableAction(dropzoneAction);

  const contextMenu = nullableAction(contextMenuAction);

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
    dispatch("highlight", entry);
  }

  function renameFile(event: Event) {
    event.preventDefault();

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

  function onDragOver({
    detail: { originalEvent },
  }: CustomEvent<{ originalEvent: DragEvent }>) {
    originalEvent.stopPropagation();

    dndHighlightedEntry = entry;
  }

  function onDrop({
    detail: { item: draggedEntry },
  }: CustomEvent<{ item: Entry<Item> }>) {
    dispatch("moved", {
      draggedEntry,
      dropzoneEntry: entry,
    });
  }
</script>

<div
  class="root"
  class:bg-gray-800={dndHighlightedEntry === entry}
  on:dblclick
  use:keyboardNavigationItem={index}
  use:dropzone={entry.subEntries.length ? { accept: type } : null}
  on:dnd-drop={onDrop}
  on:dnd-dragenter={onDragOver}
>
  <div
    class="name"
    class:font-semibold={entry.subEntries.length}
    class:lg-space={mode === "view"}
    class:highlighted-view={isHighlighted && mode === "view"}
    on:mousedown={highlight}
    use:contextMenu={withItemContextMenu}
    use:draggable={!entry.subEntries.length ? { item: entry, type } : null}
  >
    {#if entry.subEntries.length > 0}
      <div class="icon" class:expanded={isExpanded} on:click={toggleExpanded}>
        <Icon icon="ic:baseline-chevron-right" />
      </div>
    {:else}
      <div class="icon">
        <slot name="icon" {entry} />
      </div>
    {/if}
    <div class="name">
      {#if mode === "view"}
        <slot name="name" itemName={entry.name} />
      {:else}
        <form
          on:submit={renameFile}
          on:keydown={(event) => event.key === "Escape" && cancelNameEdit()}
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
    {#each entry.subEntries as subEntry (subEntry.index)}
      <div class="sub-entries">
        <svelte:self
          index={subEntry.index}
          entry={subEntry}
          {highlightedEntry}
          {withItemContextMenu}
          bind:currentlyRenameEntry
          bind:dndHighlightedEntry
          on:highlight
          on:nameEdited
          on:moved
          let:itemName
          let:entry
        >
          <slot name="icon" slot="icon" {entry} />
          <slot name="name" slot="name" {itemName} />
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
    @apply flex items-center h-7 w-full px-1 cursor-pointer border border-transparent;
  }

  .name.highlighted-view {
    @apply border border-dotted border-orange-700 bg-orange-700 bg-opacity-10;
  }

  .name.lg-space {
    @apply space-x-1;
  }

  .icon {
    @apply flex items-center text-orange-700 transition-all duration-150;
  }

  .icon.expanded {
    @apply rotate-90;
  }

  .sub-entries {
    @apply pl-2 ml-3 list-none border-l border-dotted border-gray-400;
  }
</style>
