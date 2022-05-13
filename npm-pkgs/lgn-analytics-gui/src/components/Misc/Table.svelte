<script lang="ts">
  type Key = $$Generic<string>;

  type Value = $$Generic<Record<Key, unknown>>;

  export let columns: Record<Key, string>;

  export let items: Value[];

  $: columnEntries = Object.entries(columns) as [Key, string][];
</script>

<div role="table" class="w-full text-xs text relative">
  <div
    role="rowgroup"
    class="surface h-10 w-full flex border-b border-black sticky top-24"
  >
    <div role="row" class="flex flex-row w-full">
      {#each columnEntries as [column, width], index (column)}
        <div
          role="columnheader"
          class="text-left w-full px-1"
          class:pl-4={index === 0}
          class:pl-2={index === columnEntries.length}
          style={`width: ${width}`}
        >
          <slot name="header" columnName={column} />
        </div>
      {/each}
    </div>
  </div>
  <div role="rowgroup">
    {#each items as item, index (index)}
      <div role="row" class="root flex flex-row text-sm">
        {#each columnEntries as [column, width], cellIndex (column)}
          <div
            class="border-b border-[#202020] px-1"
            class:pl-4={cellIndex === 0}
            class:pl-2={cellIndex === columnEntries.length}
            role="cell"
            style={`width: ${width}`}
          >
            <slot name="cell" columnName={column} {item} value={item[column]} />
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>
