<script lang="ts">
  import { Entry } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";
  import { extension } from "@/lib/path";
  import Icon from "@iconify/svelte";
  import { keyboardNavigationItem } from "@lgn/frontend/src/actions/keyboardNavigation";
  import TextInput from "../inputs/TextInput.svelte";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch = createEventDispatcher<{
    select: Entry<Item>;
    nameChange: { entry: Entry<Item>; newName: string };
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

  export let selectedEntry: Entry<Item> | null = null;

  export let panelIsFocused: boolean;

  // TODO: Should be in a store?
  export let currentlyRenameEntry: Entry<Item> | null = null;

  let mode: "view" | "edit";

  let isExpanded = true;

  function select() {
    dispatch("select", entry);
  }

  function renameFile(event: Event) {
    event.preventDefault();

    currentlyRenameEntry = null;

    if (nameValue.trim().length) {
      dispatch("nameChange", { entry, newName: nameValue.trim() });
    }
  }

  function cancelEdition() {
    mode = "view";
  }

  function entryName() {
    return entry.name.trim();
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  $: isSelected = selectedEntry ? entry === selectedEntry : false;

  $: nameExtension = extension(entry.name);

  $: iconName =
    (nameExtension && iconNames[nameExtension]) ||
    "ic:outline-insert-drive-file";

  $: mode =
    currentlyRenameEntry && currentlyRenameEntry === entry ? "edit" : "view";

  $: nameValue = mode === "edit" ? entryName() : "";

  $: if (!isSelected) {
    cancelEdition();
  }
</script>

<div class="root" on:dblclick use:keyboardNavigationItem={entry.index}>
  <div
    class="name"
    class:font-semibold={entry.subEntries}
    class:lg-space={mode === "view"}
    class:selected-view={isSelected && mode === "view"}
    on:mousedown={select}
  >
    {#if entry.subEntries}
      <div class="icon" class:expanded={isExpanded} on:click={toggleExpanded}>
        <Icon icon="ic:chevron-right" />
      </div>
    {:else}
      <div class="icon">
        <Icon icon={iconName} />
      </div>
    {/if}
    <div class="name">
      {#if mode === "view"}
        <slot name="name" itemName={entry.name} />
      {:else}
        <form
          on:submit={renameFile}
          on:keydown={(event) => event.key === "Escape" && cancelEdition()}
        >
          <TextInput autoFocus autoSelect size="sm" bind:value={nameValue} />
        </form>
      {/if}
    </div>
  </div>
  {#if entry.subEntries && isExpanded}
    {#each entry.subEntries || [] as entry (entry.name)}
      <div class="sub-entries">
        <svelte:self
          {entry}
          {selectedEntry}
          {panelIsFocused}
          bind:currentlyRenameEntry
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
    @apply flex flex-col cursor-pointer border-dotted border-gray-400;
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

  .sub-entries {
    @apply pl-2 ml-3 list-none border-l border-dotted border-gray-400;
  }
</style>
