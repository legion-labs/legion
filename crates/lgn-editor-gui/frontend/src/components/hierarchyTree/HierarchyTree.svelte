<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Entry, Entries } from "@/lib/hierarchyTree";
  import keyboardNavigation from "@lgn/frontend/src/actions/keyboardNavigation";
  import KeyboardNavigationStore from "@lgn/frontend/src/stores/keyboardNavigation";
  import Inner from "./Inner.svelte";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch =
    createEventDispatcher<{ highlight: Entry<Item>; select: Entry<Item> }>();

  // Can be extracted if needed
  const keyboardNavigationStore = new KeyboardNavigationStore();

  export let entries: Entries<Item>;

  export let highlightedItem: Item | null = null;

  let currentlyRenameEntry: Entry<Item> | null = null;

  $: highlightedEntry =
    entries.find((entry) => entry.item === highlightedItem) || null;

  $: $keyboardNavigationStore.currentIndex = highlightedItem
    ? entries.findIndex((entry) =>
        highlightedEntry ? entry === highlightedEntry : false
      )
    : null;

  // TODO: Use props instead of the `edit` function?
  export function edit(item: Item) {
    const entry = entries.find((entry) => entry.item === item);

    if (!entry) {
      return;
    }

    currentlyRenameEntry = entry;
  }

  // TODO: Use props instead of the `remove` function?
  export function remove(item: Item) {
    const entry = entries.find((entry) => entry.item === item);

    if (!entry) {
      return;
    }

    entries = entries.remove(entry);
  }

  function select() {
    if (!highlightedEntry) {
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
    const entry = entries.find((entry) => entry.index === index);

    if (!entry) {
      return;
    }

    setHighlightedEntry(entry);
  }
</script>

<div
  class="root"
  on:navigation-change={setHighlightedEntryWithIndex}
  on:navigation-select={select}
  use:keyboardNavigation={{
    size: entries.size,
    store: keyboardNavigationStore,
  }}
>
  {#each entries.entries as entry (entry.name)}
    <Inner
      {entry}
      {highlightedEntry}
      bind:currentlyRenameEntry
      on:dblclick={select}
      on:highlight={({ detail: entry }) => setHighlightedEntry(entry)}
      on:nameChange={setName}
      let:itemName
    >
      <slot name="name" slot="name" {itemName} />
    </Inner>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
