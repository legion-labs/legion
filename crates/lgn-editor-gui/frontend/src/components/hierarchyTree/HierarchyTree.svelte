<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Entries, isEntry } from "@/lib/hierarchyTree";
  import type { Entry } from "@/lib/hierarchyTree";
  import keyboardNavigation from "@lgn/web-client/src/actions/keyboardNavigation";
  import { createKeyboardNavigationStore } from "@lgn/web-client/src/stores/keyboardNavigation";
  import HierarchyTreeItem from "./HierarchyTreeItem.svelte";

  type Item = $$Generic<{ id: string; path: string }>;

  type $$Slots = {
    name: { itemName: string };
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

  export let entries: Entries<Item>;

  export let highlightedEntry: Entry<Item> | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let withItemContextMenu: string | null = null;

  let hierarchyTree: HTMLElement | null;

  /**
   * Currently highlighted entry _in the drag and drop context_
   * If a resource is dragged over an other resource this
   * variable will be populated by the entry that's being overed
   */
  let dndHighlightedEntry: Entry<Item> | null = null;

  $: highlightedEntry =
    entries.find((entry) => entry === highlightedEntry) || null;

  $: $keyboardNavigationStore.currentIndex = highlightedEntry
    ? entries.findIndex((entry) =>
        highlightedEntry ? entry === highlightedEntry : false
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
      updatedEntry === entry ? { ...entry, name: newName } : null
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
    highlightedEntry && startNameEdit(highlightedEntry)}
  on:navigation-remove={() =>
    highlightedEntry && removeRequest(highlightedEntry)}
  use:keyboardNavigation={{
    size: entries.size(),
    store: keyboardNavigationStore,
  }}
  bind:this={hierarchyTree}
>
  {#each entries.entries as entry (isEntry(entry) ? entry.item.id : entry.item)}
    <HierarchyTreeItem
      index={entry.index}
      {entry}
      {highlightedEntry}
      {withItemContextMenu}
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
      <svelte:fragment slot="name" let:itemName>
        <slot name="name" {itemName} />
      </svelte:fragment>
    </HierarchyTreeItem>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
