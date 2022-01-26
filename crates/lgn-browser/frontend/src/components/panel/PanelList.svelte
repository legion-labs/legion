<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { keepElementVisible } from "../../lib/html";

  type Item = $$Generic;

  type Key = keyof Item;

  interface $$Slots {
    default: { item: Item };
  }

  const dispatch = createEventDispatcher<{ select: Item }>();

  /**
   * The key attribute used to index the items during the iteration:
   * https://svelte.dev/tutorial/keyed-each-blocks
   */
  export let key: Key | null = null;

  export let items: Item[];

  export let selectedItem: Item | null;

  export let panelIsFocused: boolean;

  /**
   * This prop function is used to compare 2 items together and must return
   * `true` if the items are identical.
   *
   * By default `===` is used, so primitives are compared by value and
   * object by reference.
   */
  export let itemsAreIdentical = (item1: Item, item2: Item) => item1 === item2;

  let selectedIndex: number | null = null;

  let rootElement: HTMLDivElement | undefined;

  let itemElements: HTMLDivElement[] = [];

  $: selectedIndex = selectedItem
    ? items.findIndex((item) =>
        selectedItem ? itemsAreIdentical(item, selectedItem) : false
      )
    : null;

  function select(item: Item) {
    selectedItem = item;
  }

  function handleWindowKeyword(event: KeyboardEvent) {
    if (!panelIsFocused) {
      return;
    }

    let newIndex: number | undefined;

    switch (event.key) {
      case "ArrowUp": {
        // selectedIndex should never be lt 0
        newIndex =
          selectedIndex === null || selectedIndex <= 0
            ? items.length - 1
            : selectedIndex - 1;

        break;
      }

      case "ArrowDown": {
        // selectedIndex should never be gt `items.length - 1`
        newIndex =
          selectedIndex === null || selectedIndex >= items.length - 1
            ? 0
            : selectedIndex + 1;

        break;
      }
    }

    if (newIndex == null) {
      return;
    }

    event.preventDefault();

    if (rootElement) {
      // "Follows" the user focus when using the arrow keys
      keepElementVisible(rootElement, itemElements[newIndex]);
    }

    selectedItem = items[newIndex];

    dispatch("select", selectedItem);
  }
</script>

<svelte:window on:keydown={handleWindowKeyword} />

<div class="root" bind:this={rootElement}>
  {#each items as item, index (key ? item[key] : index)}
    <div
      class="item"
      class:selected-item={index === selectedIndex}
      class:item-panel-is-focused={panelIsFocused}
      on:mousedown={() => select(item)}
      on:dblclick
      bind:this={itemElements[index]}
    >
      <slot {item} />
    </div>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply pb-2 break-all h-full overflow-auto;
  }

  .item {
    @apply cursor-pointer hover:bg-gray-500 py-1 px-2 border border-transparent border-dotted;
  }

  .selected-item {
    @apply bg-gray-500;
  }

  .selected-item.item-panel-is-focused {
    @apply border-orange-700;
  }
</style>
