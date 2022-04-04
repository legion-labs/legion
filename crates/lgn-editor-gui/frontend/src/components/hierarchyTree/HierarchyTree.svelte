<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { v4 as uuid } from "uuid";

  import keyboardNavigation from "@lgn/web-client/src/actions/keyboardNavigation";
  import { createKeyboardNavigationStore } from "@lgn/web-client/src/stores/keyboardNavigation";

  import { Entries, isEntry } from "@/lib/hierarchyTree";
  import type { Entry, ItemBase } from "@/lib/hierarchyTree";

  import HierarchyTreeItem from "./HierarchyTreeItem.svelte";

  type Item = $$Generic<ItemBase>;

  type $$Slots = {
    name: { entry: Entry<Item | symbol> };
    icon: { entry: Entry<Item> };
  };

  const dispatch = createEventDispatcher<{
    highlight: Entry<Item>;
    select: Entry<Item>;
    nameEdited: { entry: Entry<Item>; newName: string };
    removeRequest: Entry<Item>;
  }>();

  // Can be extracted if needed
  const keyboardNavigationStore = createKeyboardNavigationStore();

  /** Unique identifier used internally, random uuid by default */
  export let id = uuid();

  export let entries: Entries<Item>;

  export let highlightedEntry: Entry<Item> | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let itemContextMenu: string | null = null;

  /** Enables entry renaming */
  export let renamable = false;

  /** Enables deletion */
  export let deletable = false;

  /**
   * Enables entry reordering using drag and drop
   * (not be mistaken with `draggable` that allows for external drag and drop)
   */
  export let reorderable = false;

  /**
   * Allows entries to be drag and droppable
   * (not be mistaken with `reorderable` that allows the entries to be reordered internally)
   */
  export let draggable: string | null = null;

  let hierarchyTree: HTMLElement | null;

  /**
   * Currently highlighted entry _in the drag and drop context_
   * If a resource is dragged over an other resource this
   * variable will be populated by the entry that's being overed
   */
  let dndHighlightedEntry: Entry<Item> | null = null;

  $: highlightedEntry =
    entries.find((entry) => entry.item.id === highlightedEntry?.item.id) ||
    null;

  $: $keyboardNavigationStore.currentIndex = highlightedEntry
    ? entries.findIndex((entry) =>
        highlightedEntry ? entry.item.id === highlightedEntry.item.id : false
      )
    : null;

  $: if (!currentlyRenameEntry) {
    focus();
  }

  function startNameEdit(entry: Entry<Item>) {
    currentlyRenameEntry = entry;
  }

  function removeRequest(entry: Entry<Item>) {
    dispatch("removeRequest", entry);
  }

  function select() {
    if (!highlightedEntry || currentlyRenameEntry) {
      return;
    }

    dispatch("select", highlightedEntry);
  }

  function setName({
    detail: { entry: updatedEntry, newName },
  }: CustomEvent<{ entry: Entry<Item>; newName: string }>) {
    entries = entries.update((entry) =>
      updatedEntry.item.id === entry.item.id
        ? { ...entry, name: newName }
        : null
    );

    dispatch("nameEdited", { entry: updatedEntry, newName });
  }

  function setHighlightedEntry(entry: Entry<Item>) {
    highlightedEntry = entry;

    if (highlightedEntry) {
      dispatch("highlight", highlightedEntry);
    }
  }

  function setHighlightedEntryWithIndex({
    detail: index,
  }: CustomEvent<number>) {
    const entry = entries.getFromIndex(index);

    if (!entry) {
      return;
    }

    setHighlightedEntry(entry);
  }

  function focus() {
    if (hierarchyTree) {
      hierarchyTree.focus();
    }
  }
</script>

<div
  class="root"
  on:navigation-change={setHighlightedEntryWithIndex}
  on:navigation-select={select}
  on:navigation-rename={() =>
    renamable && highlightedEntry && startNameEdit(highlightedEntry)}
  on:navigation-remove={() =>
    deletable && highlightedEntry && removeRequest(highlightedEntry)}
  use:keyboardNavigation={{
    size: entries.size(),
    store: keyboardNavigationStore,
  }}
  bind:this={hierarchyTree}
>
  {#each entries.entries as entry (isEntry(entry) ? entry.item.id : entry.item)}
    <HierarchyTreeItem
      {id}
      {entry}
      {highlightedEntry}
      {itemContextMenu}
      {reorderable}
      {draggable}
      index={entry.index}
      bind:currentlyRenameEntry
      bind:dndHighlightedEntry
      on:dblclick={select}
      on:highlight={({ detail: entry }) => setHighlightedEntry(entry)}
      on:nameEdited={setName}
      on:moved
    >
      <svelte:fragment slot="icon" let:entry>
        <slot name="icon" {entry} />
      </svelte:fragment>
      <svelte:fragment slot="name" let:entry>
        <slot name="name" {entry} />
      </svelte:fragment>
    </HierarchyTreeItem>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
