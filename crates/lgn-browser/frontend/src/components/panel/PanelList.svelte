<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { writable } from "svelte/store";
  import keyboardNavigation, {
    Store as KeyboardNavigationStore,
    keyboardNavigationItem,
  } from "../../actions/keyboardNavigation";

  type Item = $$Generic;

  type Key = keyof Item;

  interface $$Slots {
    default: { item: Item };
  }

  const dispatch = createEventDispatcher<{ select: Item }>();

  // Can be extracted if needed
  const keyboardNavigationStore = writable<KeyboardNavigationStore>({
    currentIndex: null,
  });

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

  $: $keyboardNavigationStore.currentIndex = selectedItem
    ? items.findIndex((item) =>
        selectedItem ? itemsAreIdentical(item, selectedItem) : false
      )
    : null;

  function setSelectedItem(item: Item) {
    selectedItem = item;

    dispatch("select", selectedItem);
  }

  function selectItemWithIndex(index: number) {
    setSelectedItem(items[index]);
  }
</script>

<div
  class="root"
  use:keyboardNavigation={{
    disabled: !panelIsFocused,
    listener: selectItemWithIndex,
    size: items.length,
    store: keyboardNavigationStore,
  }}
>
  {#each items as item, index (key ? item[key] : index)}
    <div
      class="item"
      class:selected-item={index === $keyboardNavigationStore.currentIndex}
      class:item-panel-is-focused={panelIsFocused}
      use:keyboardNavigationItem={index}
      on:mousedown={() => setSelectedItem(item)}
      on:dblclick
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
