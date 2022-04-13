<script lang="ts">
  import { createEventDispatcher } from "svelte";

  type Size = "default" | "full";

  const dispatch = createEventDispatcher<{
    change: boolean;
  }>();

  export let value: boolean;

  export let size: Size = "default";

  export let disabled = false;

  function onClick() {
    value = !value;

    dispatch("change", value);
  }
</script>

<div
  class="root"
  class:disabled
  class:default={size === "default"}
  on:click={disabled ? null : onClick}
>
  <input class="input" type="checkbox" bind:checked={value} {disabled} />
  {#if value}
    &#10003;
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex items-center justify-center cursor-pointer rounded-sm bg-gray-800;
  }

  .root.disabled {
    @apply text-gray-400;
  }

  .root.default {
    @apply h-6 w-6;
  }

  .input {
    @apply hidden;
  }
</style>
