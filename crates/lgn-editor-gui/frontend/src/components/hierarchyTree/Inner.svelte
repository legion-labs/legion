<script lang="ts">
  import { Entry } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";
  import { extension } from "@/lib/path";
  import Icon from "@iconify/svelte";
  import TextInput from "../inputs/TextInput.svelte";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch = createEventDispatcher<{
    select: Item;
    nameChange: { item: Item; newName: string };
  }>();

  // TODO: Temporary extension to icon name map, should be dynamic
  const iconNames: Record<string, string> = {
    pdf: "ic:baseline-picture-as-pdf",
    jpg: "ic:baseline-image",
    jpeg: "ic:baseline-image",
    png: "ic:baseline-image",
    zip: "ic:baseline-archive",
    rar: "ic:baseline-archive",
  };

  export let entry: Entry<Item>;

  export let selectedItem: Item | null = null;

  export let name: string;

  // TODO: This should be in a store
  export let expanded = true;

  export let itemsAreIdentical: (item1: Item, item2: Item) => boolean;

  // TODO: Should be in a store?
  export let currentlyRenameItem: Item | null = null;

  let mode: "view" | "edit";

  function toggleExpand() {
    expanded = !expanded;
  }

  function select() {
    dispatch("select", entry.item);
  }

  function renameFile(event: Event) {
    event.preventDefault();

    currentlyRenameItem = null;

    if (nameValue.trim().length) {
      dispatch("nameChange", { item: entry.item, newName: nameValue.trim() });

      nameValue = name;
    }
  }

  function cancelEdition(event?: KeyboardEvent) {
    if (event && event.key !== "Escape") {
      return;
    }

    mode = "view";

    nameValue = name;
  }

  $: isSelected = selectedItem
    ? itemsAreIdentical(entry.item, selectedItem)
    : false;

  $: nameValue = name;

  $: nameExtension = extension(name);

  $: iconName =
    (nameExtension && iconNames[nameExtension]) ||
    "ic:outline-insert-drive-file";

  $: mode =
    currentlyRenameItem && itemsAreIdentical(currentlyRenameItem, entry.item)
      ? "edit"
      : "view";

  $: if (!isSelected) {
    cancelEdition();
  }
</script>

<div class="root" on:dblclick>
  <div
    class="name"
    class:font-semibold={entry.entries}
    class:lg-space={mode === "view"}
    class:selected-view={isSelected && !entry.entries && mode === "view"}
    on:mousedown={select}
  >
    {#if entry.entries}
      <div class="icon" class:expanded on:click={toggleExpand}>
        <Icon icon="ic:chevron-right" />
      </div>
    {:else}
      <div class="icon">
        <Icon icon={iconName} />
      </div>
    {/if}
    <div class="name">
      {#if mode === "view"}
        <slot name="name" itemName={name} />
      {:else}
        <form on:submit={renameFile} on:keydown={cancelEdition}>
          <TextInput autoFocus autoSelect size="sm" bind:value={nameValue} />
        </form>
      {/if}
    </div>
  </div>
  {#if entry.entries && expanded}
    {#each Object.entries(entry.entries || {}) as [name, entry] (name)}
      <div class="files">
        <svelte:self
          {entry}
          {selectedItem}
          {name}
          {itemsAreIdentical}
          bind:currentlyRenameItem
          on:select
          on:nameChange
          let:itemName
        >
          <slot name="name" slot="name" {itemName} />
        </svelte:self>
      </div>
    {/each}
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col cursor-pointer;
  }

  .name {
    @apply flex items-center h-7 w-full px-1 cursor-pointer border border-transparent;
  }

  .name.selected-view {
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

  .files {
    @apply pl-2 ml-1.5 list-none border-l border-dotted border-gray-400;
  }
</style>
