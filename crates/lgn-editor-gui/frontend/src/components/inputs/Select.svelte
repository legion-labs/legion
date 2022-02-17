<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Icon from "@iconify/svelte";
  import clickOutside from "@lgn/web-client/src/actions/clickOutside";
  import keyboardNavigation, {
    keyboardNavigationItem,
    keyboardNavigationContainer,
  } from "@lgn/web-client/src/actions/keyboardNavigation";
  import KeyboardNavigationStore from "@lgn/web-client/src/stores/keyboardNavigation";

  type Item = $$Generic;

  const dispatch = createEventDispatcher<{
    change: Option | "";
  }>();

  type Option = { value: string; item: Item };

  const keyboardNavigationStore = new KeyboardNavigationStore();

  export let status: "default" | "error" = "default";

  export let size: "default" | "lg" = "default";

  export let options: Option[];

  export let disabled = false;

  export let value: Option | "";

  let isOpen = false;

  let highlightedOptionIndex: number | null = null;

  function toggle() {
    if (disabled) {
      return;
    }

    isOpen ? close() : open();
  }

  function open() {
    isOpen = true;
  }

  function close() {
    isOpen = false;

    highlightedOptionIndex = null;
  }

  function select(option: Option | "") {
    value = option;
    if (option != "") {
      dispatch("change", option);
    } else {
    }

    close();
  }

  function selectCurrentlyHighlightedItem() {
    if (highlightedOptionIndex === null) {
      return;
    }

    select(
      options[
        $$slots.unselect ? highlightedOptionIndex - 1 : highlightedOptionIndex
      ]
    );
  }

  function setHighlightedItem(index: number | null) {
    highlightedOptionIndex = index;

    $keyboardNavigationStore.currentIndex = highlightedOptionIndex;
  }

  function setHighlightedItemWithIndex({ detail: index }: CustomEvent<number>) {
    setHighlightedItem(index);
  }

  $: if (disabled) {
    close();
  }
</script>

<div
  class="root"
  class:disabled
  class:default={size === "default"}
  class:lg={size === "lg"}
  class:error={status === "error" && !disabled}
>
  <div
    class="select"
    on:navigation-change={setHighlightedItemWithIndex}
    on:navigation-select={selectCurrentlyHighlightedItem}
    on:click-outside={close}
    use:clickOutside
    use:keyboardNavigation={{
      size: options.length + ($$slots.unselect ? 1 : 0),
      store: keyboardNavigationStore,
    }}
  >
    <div class="selected-label" on:click={toggle}>
      <div>
        {#if value}
          <div><slot name="option" option={value} /></div>
        {:else}
          <div>
            <slot name="label">
              <div />
            </slot>
          </div>
        {/if}
      </div>
      <div class="icon">
        <Icon icon="ic:baseline-keyboard-arrow-down" />
      </div>
    </div>
    <!-- on:mouseenter={resetHighlightedOptionIndex} -->
    <div class="options" class:hidden={!isOpen} use:keyboardNavigationContainer>
      {#if $$slots.unselect}
        <div
          class="option"
          class:bg-gray-500={highlightedOptionIndex === 0}
          on:mousemove={() => setHighlightedItem(0)}
          on:click={() => select("")}
          use:keyboardNavigationItem={0}
        >
          <slot name="unselect" />
        </div>
      {/if}
      {#each options as option, index (option.value)}
        {@const actualIndex = $$slots.unselect ? index + 1 : index}

        <div
          class="option"
          class:bg-gray-500={highlightedOptionIndex === actualIndex}
          on:mousemove={() => setHighlightedItem(actualIndex)}
          on:click={() => select(option)}
          use:keyboardNavigationItem={actualIndex}
        >
          <slot name="option" {option} />
        </div>
      {/each}
    </div>
  </div>
</div>

<select class="hidden" {disabled} value>
  {#if $$slots.unselect}
    <option value="">
      <slot name="unselect" />
    </option>
  {/if}
  {#each options as option (option.value)}
    <option value={option.value}>
      <slot {option} />
    </option>
  {/each}
</select>

<style lang="postcss">
  .root {
    @apply w-full bg-gray-800 rounded-sm cursor-pointer;
  }

  .root.disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .root.default {
    @apply h-8;
  }

  .root.lg {
    @apply h-10;
  }

  .root.error {
    @apply border border-red-700;
  }

  .select {
    @apply relative w-full h-full bg-gray-800 rounded-sm outline-none appearance-none;
  }

  .selected-label {
    @apply flex flex-row h-full w-full p-2 justify-between;
  }

  .disabled .icon {
    @apply text-gray-400;
  }

  .icon {
    @apply flex items-center h-full text-orange-700;
  }

  .options {
    @apply max-h-44 overflow-auto absolute py-1 mt-1 bg-gray-800 rounded-sm w-full shadow-lg shadow-gray-800;
  }

  .option {
    @apply flex items-center h-8 px-2;
  }
</style>
