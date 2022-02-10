<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Entry, Entries } from "@/lib/hierarchyTree";
  import keyboardNavigation from "@lgn/web-client/src/actions/keyboardNavigation";
  import KeyboardNavigationStore from "@lgn/web-client/src/stores/keyboardNavigation";
  import Inner from "./Inner.svelte";

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
  use:keyboardNavigation={{
    size: entries.size,
    store: keyboardNavigationStore,
  }}
  bind:this={hierarchyTree}
>
  {#each entries.entries as entry (entry.index)}
    <Inner
      index={entry.index}
      {entry}
      {highlightedEntry}
      bind:currentlyRenameEntry
      on:dblclick={select}
      on:highlight={({ detail: entry }) => setHighlightedEntry(entry)}
      on:nameEdited={setName}
    >
      <slot name="name" slot="name" let:itemName {itemName} />
    </Inner>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
