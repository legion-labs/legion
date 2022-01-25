<script lang="ts">
  import { Entries, updateEntry } from "@/lib/hierarchyTree";
  import clickOutside from "@lgn/frontend/src/actions/clickOutside";
  import { createEventDispatcher } from "svelte";
  import Inner from "./Inner.svelte";

  type Item = $$Generic;

  type $$Slots = {
    itemName: { itemName: string };
  };

  const dispatch = createEventDispatcher<{ select: Item }>();

  export let entries: Entries<Item>;

  export let activeItem: Item | null = null;

  /**
   * This prop function is used to compare 2 items together and must retur
   * `true` if the items are identical.
   *
   * By default `===` is used, so primitives are compared by value and
   * object by reference.
   */
  export let itemsAreIdentical = (item1: Item, item2: Item) => item1 === item2;

  function setActiveItem({ detail: item }: CustomEvent<Item>) {
    activeItem = item;

    dispatch("select", item);
  }

  function setName({
    detail: { item, newName },
  }: CustomEvent<{ newName: string; item: Item }>) {
    entries = updateEntry(entries, (_name, otherItem) =>
      itemsAreIdentical(otherItem, item) ? { name: newName } : null
    );
  }
</script>

<div class="root" use:clickOutside={() => (activeItem = null)}>
  {#each Object.entries(entries) as [name, entry] (name)}
    <Inner
      {entry}
      {activeItem}
      {itemsAreIdentical}
      {name}
      on:select={setActiveItem}
      on:nameChange={setName}
      let:itemName
    >
      <slot name="itemName" slot="itemName" {itemName} />
    </Inner>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-2 overflow-y-auto;
  }
</style>
