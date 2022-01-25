<script lang="ts">
  import { Entry } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";
  import { extension } from "@/lib/path";
  import Icon from "@iconify/svelte";
  import TextInput from "../inputs/TextInput.svelte";

  type Item = $$Generic;

  type $$Slots = {
    itemName: { itemName: string };
  };

  const dispatch = createEventDispatcher<{
    select: Item;
    nameChange: { item: Item; newName: string };
  }>();

  // TODO: Temporary extension to icon name map, should be dynamic
  const iconNames: Record<string, string> = {
    pdf: "mdi:file-pdf-box",
    jpg: "mdi:file-image",
    jpeg: "mdi:file-image",
    png: "mdi:file-image",
    zip: "mdi:file-cabinet",
    rar: "mdi:file-cabinet",
  };

  export let entry: Entry<Item>;

  export let activeItem: Item | null = null;

  export let name: string;

  // TODO: This should be in a store
  export let expanded = true;

  export let itemsAreIdentical: (item1: Item, item2: Item) => boolean;

  let mode: "view" | "edit" = "view";

  function toggleExpand() {
    expanded = !expanded;
  }

  function select() {
    dispatch("select", entry.item);
  }

  function edit() {
    if (mode === "view") {
      mode = "edit";
    }
  }

  function renameFile(event: Event) {
    event.preventDefault();

    mode = "view";

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

  $: isActive = activeItem ? itemsAreIdentical(entry.item, activeItem) : false;

  $: nameValue = name;

  $: nameExtension = extension(name);

  $: iconName =
    (nameExtension && iconNames[nameExtension]) || "mdi:file-outline";

  $: if (!isActive) {
    cancelEdition();
  }
</script>

<div class="root">
  <div
    class="name"
    class:font-semibold={entry.entries}
    class:lg-space={mode === "view"}
    class:active-view={isActive && !entry.entries && mode === "view"}
    on:click={select}
    on:dblclick={edit}
  >
    {#if entry.entries}
      <div class="icon" class:expanded on:click={toggleExpand}>
        <Icon icon="mdi:chevron-right" />
      </div>
    {:else}
      <div class="icon">
        <Icon icon={iconName} />
      </div>
    {/if}
    <div class="name">
      {#if mode === "view"}
        <slot name="itemName" itemName={name} />
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
          {activeItem}
          {name}
          {itemsAreIdentical}
          on:select
          on:nameChange
          let:itemName
        >
          <slot name="itemName" slot="itemName" {itemName} />
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

  .name.active-view {
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
