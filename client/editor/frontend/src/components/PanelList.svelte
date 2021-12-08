<script lang="ts">
  import { keepElementVisible } from "@/lib/html";
  import { createEventDispatcher, getContext } from "svelte";
  import { Writable } from "svelte/store";
  import { panelIsFocusedContext } from "./Panel.svelte";

  type Item = $$Generic;

  type Key = keyof Item;

  interface $$Slots {
    default: { item: Item };
  }

  type ItemChangeEventDetail = {
    direction: "up" | "down";
    newIndex: number;
    newItem: Item;
  };

  const dispatch = createEventDispatcher<{
    click: Item;
    itemChange: ItemChangeEventDetail;
  }>();

  const panelIsFocused = getContext<Writable<boolean>>(panelIsFocusedContext);

  /**
   * The key attribute used to index the items during the iteration:
   * https://svelte.dev/tutorial/keyed-each-blocks
   */
  export let key: Key | null = null;

  export let items: Item[];

  export let activeItem: Item | null;

  let activeIndex = -1;

  let rootElement: HTMLDivElement | undefined;

  let itemElements: HTMLDivElement[] = [];

  $: if ($panelIsFocused) {
    activeIndex = activeItem ? items.indexOf(activeItem) : -1;
  }

  function handleWindowKeyword(event: KeyboardEvent) {
    if (!$panelIsFocused) {
      return;
    }

    let eventDetail: ItemChangeEventDetail | null = null;

    switch (event.key) {
      case "ArrowUp": {
        const newIndex = activeIndex > 0 ? activeIndex - 1 : items.length - 1;

        eventDetail = {
          direction: "up",
          newIndex,
          newItem: items[newIndex],
        };

        break;
      }

      case "ArrowDown": {
        const newIndex =
          activeIndex > -1 && activeIndex < items.length - 1
            ? activeIndex + 1
            : 0;

        eventDetail = {
          direction: "down",
          newIndex,
          newItem: items[newIndex],
        };

        break;
      }
    }

    if (!eventDetail) {
      return;
    }

    event.preventDefault();

    if (rootElement) {
      // "Follows" the user focus when using the arrow keys
      keepElementVisible(rootElement, itemElements[eventDetail.newIndex]);
    }

    dispatch("itemChange", eventDetail);
  }
</script>

<svelte:window on:keydown={handleWindowKeyword} />

<div class="root" bind:this={rootElement}>
  {#each items as item, index (key ? item[key] : index)}
    <div
      class="item"
      class:active-item={index === activeIndex}
      class:item-panel-is-focused={$panelIsFocused}
      on:click={() => dispatch("click", item)}
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

  .active-item {
    @apply bg-gray-500;
  }

  .active-item.item-panel-is-focused {
    @apply border-gray-800;
  }
</style>
