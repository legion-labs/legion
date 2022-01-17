<script context="module" lang="ts">
  export type Entries<Item> = (
    | { type: "file"; name: string; item: Item }
    | { type: "directory"; name: string; entries: Entries<Item> }
  )[];
</script>

<script lang="ts">
  import clickOutside from "@lgn/frontend/src/actions/clickOutside";

  import Folder from "./Folder.svelte";

  type Item = $$Generic;

  export let rootName: string;

  export let entries: Entries<Item>;

  export let activeItem: Item | null = null;

  function setActiveItem(event: CustomEvent<Item>) {
    activeItem = event.detail;
  }

  // TODO: Improve performance if needed
  function updateEntries(
    entries: Entries<Item>,
    updatedItem: Item,
    newName: string
  ): Entries<Item> {
    return entries.map((entry) => {
      switch (entry.type) {
        case "directory": {
          return {
            ...entry,
            entries: updateEntries(entry.entries, updatedItem, newName),
          };
        }

        case "file": {
          if (entry.item === updatedItem) {
            return { ...entry, name: newName };
          } else {
            return entry;
          }
        }
      }
    });
  }

  function setName({
    detail: { item, newName },
  }: CustomEvent<{ newName: string; item: Item }>) {
    entries = updateEntries(entries, item, newName);
  }
</script>

<div class="root" use:clickOutside={() => (activeItem = null)}>
  <Folder
    name={rootName}
    {entries}
    {activeItem}
    on:select={setActiveItem}
    on:nameChange={setName}
  />
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
