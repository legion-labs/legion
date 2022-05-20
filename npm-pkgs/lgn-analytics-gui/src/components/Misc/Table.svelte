<script lang="ts">
  type Name = $$Generic<string>;

  type Item = $$Generic;

  export let columns: { name: Name; width: string }[];

  export let items: Item[];

  export let customKey: (item: Item, index: number) => string | number = (
    _value,
    index
  ) => index;

  /** Force the table header to be sticky. The provided value is in rem. */
  export let sticky: number | null = null;

  $: normalizedColumns = columns.reduce(
    (acc, { name, width }) => ({ ...acc, [name]: width }),
    {} as Record<Name, string>
  );
</script>

<div role="table" class="w-full text-xs text relative">
  {#if $$slots.header}
    <div
      role="rowgroup"
      class="surface h-10 w-full flex border-b border-black z-10"
      class:sticky
      style={sticky !== null ? `top: ${sticky}rem;` : null}
    >
      <div role="row" class="flex flex-row w-full">
        {#each columns as { name: columnName, width }, index (columnName)}
          <div
            role="columnheader"
            class="text-left w-full px-1"
            class:pl-4={index === 0}
            class:pl-2={index === columns.length}
            style={`width: ${width}`}
          >
            <slot name="header" {columnName} />
          </div>
        {/each}
      </div>
    </div>
  {/if}
  <div class="w-full" role="rowgroup">
    {#each items as item, index (customKey(item, index))}
      <slot name="row" {index} {item} {columns} {normalizedColumns}>
        <div role="row" class="root flex flex-row text-sm">
          {#each columns as { name: columnName, width }, cellIndex (columnName)}
            <div
              class="border-b border-[#202020] px-1"
              class:pl-4={cellIndex === 0}
              class:pl-2={cellIndex === columns.length}
              role="cell"
              style={`width: ${width}`}
            >
              <slot name="cell" {columnName} {item} {index} />
            </div>
          {/each}
        </div>
      </slot>
    {/each}
  </div>
</div>
