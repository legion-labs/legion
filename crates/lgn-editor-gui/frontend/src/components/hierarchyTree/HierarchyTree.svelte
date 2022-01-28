<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { writable } from "svelte/store";
  import { Entry, Entries } from "@/lib/hierarchyTree";
  import keyboardNavigation, {
    Store as KeyboardNavigationStore,
  } from "@lgn/frontend/src/actions/keyboardNavigation";
  import Inner from "./Inner.svelte";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch = createEventDispatcher<{ select: Entry<Item> }>();

  // Can be extracted if needed
  const keyboardNavigationStore = writable<KeyboardNavigationStore>({
    currentIndex: null,
  });

  export let entries: Entries<Item>;

  export let selectedItem: Item | null = null;

  export let panelIsFocused: boolean;

  let currentlyRenameEntry: Entry<Item> | null = null;

  $: selectedEntry =
    entries.find((entry) => entry.item === selectedItem) || null;

  $: $keyboardNavigationStore.currentIndex = selectedItem
    ? entries.findIndex((entry) =>
        selectedEntry ? entry === selectedEntry : false
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

  function setName({
    detail: { entry: updatedEntry, newName },
  }: CustomEvent<{ entry: Entry<Item>; newName: string }>) {
    entries = entries.update((entry) =>
      updatedEntry === entry ? { ...entry, name: newName } : null
    );
  }

  function setSelectedEntry(entry: Entry<Item>) {
    selectedItem = entry.item;

    if (selectedEntry) {
      dispatch("select", selectedEntry);
    }
  }

  function setSelectEntryWithIndex(index: number) {
    const entry = entries.find((entry) => entry.index === index);

    if (!entry) {
      return;
    }

    setSelectedEntry(entry);
  }
</script>

<div
  class="root"
  use:keyboardNavigation={{
    disabled: !panelIsFocused,
    listener: setSelectEntryWithIndex,
    size: entries.size,
    store: keyboardNavigationStore,
  }}
>
  {#each entries.entries as entry (entry.name)}
    <Inner
      {entry}
      {selectedEntry}
      {panelIsFocused}
      bind:currentlyRenameEntry
      on:dblclick
      on:select={({ detail: entry }) => setSelectedEntry(entry)}
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
