<script lang="ts">
  import { Entry } from "@/lib/hierarchyTree";
  import { createEventDispatcher } from "svelte";
  import { extension } from "@/lib/path";
  import Icon from "@iconify/svelte";
  import { keyboardNavigationItem } from "@lgn/web-client/src/actions/keyboardNavigation";
  import contextMenuAction from "@/actions/contextMenu";
  import TextInput from "../inputs/TextInput.svelte";
  import sortable from "@lgn/web-client/src/actions/sortable";

  type Item = $$Generic;

  type $$Slots = {
    name: { itemName: string };
  };

  const dispatch = createEventDispatcher<{
    highlight: Entry<Item>;
    nameEdited: { entry: Entry<Item>; newName: string };
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

  export let index: number;

  export let entry: Entry<Item>;

  export let highlightedEntry: Entry<Item> | null = null;

  export let currentlyRenameEntry: Entry<Item> | null = null;

  export let withItemContextMenu: string | null = null;

  let mode: "view" | "edit";

  let isExpanded = true;

  $: isHighlighted = highlightedEntry ? entry === highlightedEntry : false;

  $: nameExtension = extension(entry.name)?.toLowerCase();

  $: iconName =
    (nameExtension && iconNames[nameExtension]) ||
    "ic:outline-insert-drive-file";

  $: mode =
    currentlyRenameEntry && currentlyRenameEntry === entry ? "edit" : "view";

  $: nameValue = mode === "edit" ? entryName() : "";

  $: if (!isHighlighted) {
    cancelNameEdit();
  }

  function extractAutoSelectRange() {
    const name = entryName();

    const ext = extension(name);

    if (ext == null) {
      return true;
    }

    return [0, name.length - ext.length - 1] as const;
  }

  function highlight() {
    dispatch("highlight", entry);
  }

  function renameFile(event: Event) {
    event.preventDefault();

    currentlyRenameEntry = null;

    if (nameValue.trim().length) {
      dispatch("nameEdited", { entry, newName: nameValue.trim() });
    }
  }

  function cancelNameEdit() {
    mode = "view";
  }

  function entryName() {
    return entry.name.trim();
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  // Simple wrapper for the `contextMenu` action that handles `null` values
  function contextMenu(element: HTMLElement) {
    if (withItemContextMenu == null) {
      return;
    }

    return contextMenuAction(element, withItemContextMenu);
  }
</script>

<div
  class="root"
  on:dblclick
  use:keyboardNavigationItem={index}
  data-not-draggable={!!entry.subEntries}
>
  <div
    class="name"
    class:font-semibold={entry.subEntries}
    class:lg-space={mode === "view"}
    class:highlighted-view={isHighlighted && mode === "view"}
    on:mousedown={highlight}
    use:contextMenu
  >
    {#if entry.subEntries}
      <div class="icon" class:expanded={isExpanded} on:click={toggleExpanded}>
        <Icon icon="ic:baseline-chevron-right" />
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
          on:keydown={(event) => event.key === "Escape" && cancelNameEdit()}
        >
          <TextInput
            autoFocus
            autoSelect={extractAutoSelectRange()}
            size="sm"
            bind:value={nameValue}
          />
        </form>
      {/if}
    </div>
  </div>
  {#if entry.subEntries && isExpanded}
    {#each entry.subEntries || [] as subEntry (subEntry.index)}
      <div
        class="sub-entries"
        use:sortable={{
          group: "nested",
          animation: 150,
          fallbackOnBody: true,
          swapThreshold: 0.65,
          filter: '[data-not-draggable="true"]',
        }}
      >
        <svelte:self
          index={subEntry.index}
          entry={subEntry}
          {highlightedEntry}
          {withItemContextMenu}
          bind:currentlyRenameEntry
          on:highlight
          on:nameEdited
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

  .name.highlighted-view {
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
