<script context="module" lang="ts">
  export const panelIsFocusedContext = "panelIsFocused";
</script>

<script lang="ts">
  import { setContext } from "svelte";
  import clickOutside from "@/actions/clickOutside";
  import { writable } from "svelte/store";

  export let isFocused = writable(false);

  setContext(panelIsFocusedContext, isFocused);
</script>

<div
  class="root"
  on:click={() => ($isFocused = true)}
  use:clickOutside={() => ($isFocused = false)}
>
  <div class="tabs">
    <div class="header">
      <div class="header-container">
        <slot name="header" />
      </div>
    </div>
    <div class="tabs-filler-bg">
      <div class="tabs-filler" />
    </div>
  </div>
  <div class="content">
    <slot name="content" />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col w-full h-full;
  }

  .tabs {
    @apply flex flex-row flex-shrink-0 h-8;
  }

  .header {
    @apply flex bg-black flex-shrink-0 rounded-tl-lg;
  }

  .header-container {
    @apply flex flex-row items-center bg-gray-700 px-2 rounded-t-lg;
  }

  .tabs-filler-bg {
    @apply flex bg-gray-700 w-full rounded-tr-lg;
  }

  .tabs-filler {
    @apply bg-black rounded-tr-lg rounded-bl-lg w-full;
  }

  .content {
    @apply bg-gray-700 flex-1 w-full rounded-b-lg overflow-hidden;
  }
</style>
