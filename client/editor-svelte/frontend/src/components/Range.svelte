<script lang="ts">
  import NumberInput from "./NumberInput.svelte";

  export let value: number;

  export let fullWidth = false;

  export let withNumberInput = false;

  export let min = -10;

  export let max = 10;
</script>

<div class="root" class:w-full={fullWidth}>
  <div class="slider-container group">
    <div>{min}</div>
    <input class="slider" type="range" {min} {max} bind:value />
    <div>{max}</div>
  </div>
  {#if withNumberInput}
    <div class="numeric-input-container">
      <NumberInput bind:value {min} {max} noArrow fullWidth />
    </div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex h-7 space-x-4 items-center;
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
