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

<div class="root group" class:disabled on:click={disabled ? null : toggle}>
  <div
    class="handler"
    class:disabled
    class:handler-off={!value}
    class:handler-on={value}
  />
</div>

<style lang="postcss">
  .root {
    @apply flex h-7 w-12 rounded-full bg-gray-800 items-center px-0.5 cursor-pointer;
  }

  .disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .handler {
    @apply h-6 w-6 rounded-full bg-gray-700 group-hover:bg-gray-500 transition-all;
  }

  .handler.disabled {
    @apply group-hover:bg-gray-400;
  }

  .handler-off {
    @apply ml-0;
  }

  .handler-on {
    @apply ml-5 bg-gray-400;
  }
</style>
