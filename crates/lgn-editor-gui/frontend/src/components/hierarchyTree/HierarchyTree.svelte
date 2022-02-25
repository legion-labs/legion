<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Entry, Entries } from "@/lib/hierarchyTree";
  import keyboardNavigation from "@lgn/web-client/src/actions/keyboardNavigation";
  import KeyboardNavigationStore from "@lgn/web-client/src/stores/keyboardNavigation";
  import HierarchyTreeItem from "./HierarchyTreeItem.svelte";
  import sortable from "@lgn/web-client/src/actions/sortable";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch = createEventDispatcher<{
    highlight: Entry<Item>;
    select: Entry<Item>;
    nameEdited: { entry: Entry<Item>; newName: string };
    removed: Entry<Item>;
  }>();

  // Can be extracted if needed
  const keyboardNavigationStore = new KeyboardNavigationStore();

  export let entries: Entries<Item>;

  export let highlightedItem: Item | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let withItemContextMenu: string | null = null;

  let hierarchyTree: HTMLElement | null;

  $: highlightedEntry =
    entries.find((entry) => entry.item === highlightedItem) || null;

  $: $keyboardNavigationStore.currentIndex = highlightedItem
    ? entries.findIndex((entry) =>
        highlightedEntry ? entry === highlightedEntry : false
      )
    : null;

  $: if (!currentlyRenameEntry) {
    focus();
  }

  export function startNameEdit(item: Item) {
    const entry = entries.find((entry) => entry.item === item);

    if (!entry) {
      return;
    }

    currentlyRenameEntry = entry;
  }

  export function remove(item: Item) {
    const entry = entries.find((entry) => entry.item === item);

    if (!entry) {
      return;
    }

    entries = entries.remove(entry);

    dispatch("removed", entry);
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
    highlightedItem = entry.item;

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
  on:navigation-rename={() => highlightedItem && startNameEdit(highlightedItem)}
  on:navigation-remove={() => highlightedItem && remove(highlightedItem)}
  bind:this={hierarchyTree}
  use:keyboardNavigation={{
    size: entries.size(),
    store: keyboardNavigationStore,
  }}
  use:sortable={{
    group: "nested",
    animation: 150,
    fallbackOnBody: true,
    swapThreshold: 0.65,
    filter: "[data-not-draggable]",
  }}
>
  {#each entries.entries as entry (entry.index)}
    <HierarchyTreeItem
      index={entry.index}
      {entry}
      {highlightedEntry}
      {withItemContextMenu}
      bind:currentlyRenameEntry
      on:dblclick={select}
      on:highlight={({ detail: entry }) => setHighlightedEntry(entry)}
      on:nameEdited={setName}
    >
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

  :global(.highlight) {
    @apply bg-gray-800;
  }
</style>
