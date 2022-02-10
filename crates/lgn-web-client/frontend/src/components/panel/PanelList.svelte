<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import keyboardNavigation, {
    keyboardNavigationItem,
  } from "../../actions/keyboardNavigation";
  import KeyboardNavigationStore from "../../stores/keyboardNavigation";

  type Item = $$Generic;

  type Key = keyof Item;

  interface $$Slots {
    default: { item: Item };
  }

  const dispatch = createEventDispatcher<{ highlight: Item; select: Item }>();

  // Can be extracted if needed
  const keyboardNavigationStore = new KeyboardNavigationStore();

  /**
   * The key attribute used to index the items during the iteration:
   * https://svelte.dev/tutorial/keyed-each-blocks
   */
  export let key: Key | null = null;

  export let items: Item[];

  export let highlightedItem: Item | null;

  export let panelIsFocused: boolean;

  /**
   * This prop function is used to compare 2 items together and must return
   * `true` if the items are identical.
   *
   * By default `===` is used, so primitives are compared by value and
   * object by reference.
   */
  export let itemsAreIdentical = (item1: Item, item2: Item) => item1 === item2;

  $: $keyboardNavigationStore.currentIndex = highlightedItem
    ? items.findIndex((item) =>
        highlightedItem ? itemsAreIdentical(item, highlightedItem) : false
      )
    : null;

  function select() {
    if (!highlightedItem) {
      return;
    }

    dispatch("select", highlightedItem);
  }

  function setHighlightedItem(item: Item) {
    highlightedItem = item;

    dispatch("highlight", highlightedItem);
  }

  function highlightItemWithIndex({ detail: index }: CustomEvent<number>) {
    setHighlightedItem(items[index]);
  }
</script>

<div
  class="root"
  on:navigation-change={highlightItemWithIndex}
  on:navigation-select={select}
  use:keyboardNavigation={{
    size: items.length,
    store: keyboardNavigationStore,
  }}
>
  {#each items as item, index (key ? item[key] : index)}
    <div
      class="item"
      class:highlighted-item={index === $keyboardNavigationStore.currentIndex}
      class:item-panel-is-focused={panelIsFocused}
      use:keyboardNavigationItem={index}
      on:mousedown={() => setHighlightedItem(item)}
      on:dblclick={select}
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

  .highlighted-item {
    @apply bg-gray-500;
  }

  .highlighted-item.item-panel-is-focused {
    @apply border-orange-700;
  }
</style>
