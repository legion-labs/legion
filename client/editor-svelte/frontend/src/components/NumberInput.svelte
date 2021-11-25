<script lang="ts">
  export let value: number;

  export let min: number | undefined = undefined;

  export let max: number | undefined = undefined;

  export let size: "sm" | "default" = "default";

  export let fullWidth = false;

  export let noArrow = false;
</script>

<input
  class="input"
  class:sm={size === "sm"}
  class:default={size === "default"}
  class:w-full={fullWidth}
  class:no-arrow={noArrow}
  type="number"
  {min}
  {max}
  bind:value
/>

<style lang="postcss">
  .input {
    @apply bg-gray-800 border-gray-400 px-2 py-1 rounded-sm outline-none;
  }

  .sm {
    @apply h-7 text-sm;
  }

  .default {
    @apply h-8;
  }
</style>
