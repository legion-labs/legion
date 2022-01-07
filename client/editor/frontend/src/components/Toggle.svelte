<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{
    input: boolean;
  }>();

  export let value: boolean;

  function toggle() {
    value = !value;
    dispatch("input", value);
  }
</script>

<div class="root group" on:click={toggle}>
  <div class="handler" class:handler-off={!value} class:handler-on={value} />
</div>

<style lang="postcss">
  .root {
    @apply flex h-7 w-12 rounded-full bg-gray-800 items-center px-0.5 cursor-pointer;
  }

  .handler {
    @apply h-6 w-6 rounded-full bg-gray-700 group-hover:bg-gray-500 transition-all;
  }

  .handler-off {
    @apply ml-0;
  }

  .handler-on {
    @apply ml-5 bg-gray-400;
  }
</style>
