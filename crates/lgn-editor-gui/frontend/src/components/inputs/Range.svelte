<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import NumberInput from "./NumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: number }>();

  export let value: number;

  /** Basically an `width: 100%` style so that the parent can control the width */
  export let fluid = false;

  export let withNumberInput = false;

  export let min = -10;

  export let max = 10;

  export let disabled = false;

  function onRangeInput(
    event: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    // Svelte will not call this function if the input value
    // is not a valid number, so we can safely cast it to `number`
    dispatch("input", +event.currentTarget.value);
  }

  function onNumberInput({ detail }: CustomEvent<number>) {
    dispatch("input", detail);
  }
</script>

<div class="root" class:disabled class:w-full={fluid}>
  <div class="slider-container group">
    <div>{min}</div>
    <input
      class="slider"
      type="range"
      {min}
      {max}
      on:input={disabled ? null : onRangeInput}
      bind:value
      {disabled}
    />
    <div>{max}</div>
  </div>
  {#if withNumberInput}
    <div class="numeric-input-container">
      <NumberInput
        bind:value
        {min}
        {max}
        on:input={onNumberInput}
        noArrow
        fluid
        {disabled}
      />
    </div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex h-7 space-x-4 items-center;
  }

  .root.disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .slider-container {
    @apply flex items-center space-x-2 w-2/3 flex-grow-0;
  }

  .slider {
    @apply bg-gray-800 h-1 border-none rounded-full w-full appearance-none;
  }

  .slider::-webkit-slider-thumb {
    @apply bg-gray-500 hover:bg-gray-400 w-4 h-4 group-hover:h-6 group-hover:w-6 cursor-pointer border-none rounded-full transition-all appearance-none;
  }

  .slider::-moz-range-thumb {
    @apply bg-gray-500 hover:bg-gray-400 w-4 h-4 group-hover:h-6 group-hover:w-6 cursor-pointer border-none rounded-full transition-all;
  }

  .numeric-input-container {
    @apply flex-grow-0 w-12;
  }
</style>
