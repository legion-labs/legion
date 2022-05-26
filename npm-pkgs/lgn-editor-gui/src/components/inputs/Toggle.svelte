<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{
    input: boolean;
  }>();

  export let value: boolean;

  export let disabled = false;

  function toggle() {
    value = !value;
    dispatch("input", value);
  }
</script>

<div
  class="root group"
  class:enabled={value}
  class:disabled
  on:click={disabled ? null : toggle}
>
  <div
    class="handler"
    class:disabled
    class:handler-off={!value}
    class:handler-on={value}
  />
</div>

<style lang="postcss">
  .root {
    @apply flex h-[16px] w-[26px] rounded-full bg-surface-500 items-center px-0.5 cursor-pointer;
  }

  .root.enabled {
    @apply bg-orange-700;
  }

  .disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .handler {
    /* group-hover:bg-orange-700 */
    @apply h-4 w-4 rounded-full transition-all;
  }

  /* .handler.disabled {
    @apply group-hover:bg-gray-400;
  } */

  .handler-off {
    @apply ml-[2px] bg-item-max;
  }

  .handler-on {
    @apply ml-[10px] bg-item-max;
  }
</style>
