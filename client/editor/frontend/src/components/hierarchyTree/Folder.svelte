<script lang="ts">
  import Icon from "../Icon.svelte";
  import File from "./File.svelte";
  import { Entries } from "./HierarchyTree.svelte";

  type Item = $$Generic;

  export let activeItem: Item | null = null;

  export let expanded = false;

  export let name: string;

  export let entries: Entries<Item>;

  function toggleExpand() {
    expanded = !expanded;
  }
</script>

<div class="root">
  <div class="name" on:click={toggleExpand}>
    <div class="icon">
      <Icon name={expanded ? "open-folder" : "closed-folder"} />
    </div>
    <div>{name}</div>
  </div>
  {#if expanded}
    <ul class="files">
      {#each entries as entry}
        <li>
          {#if entry.type === "directory"}
            <svelte:self
              name={entry.name}
              entries={entry.entries}
              {activeItem}
              on:select
              on:nameChange
            />
          {:else}
            <File
              name={entry.name}
              item={entry.item}
              isActive={activeItem === entry.item}
              on:select
              on:nameChange
            />
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style lang="postcss">
  .name {
    @apply flex items-center space-x-1 py-0.5 cursor-pointer font-bold;
  }

  .icon {
    @apply flex items-center text-orange-700;
  }

  .files {
    @apply pl-2 ml-1.5 list-none border-l border-dotted border-gray-400;
  }
</style>
