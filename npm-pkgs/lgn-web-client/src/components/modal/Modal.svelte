<!--
@Component

_Not to be mistaken with the `ModalContainer` component._

Simple Modal component wrapper. You can provided a title, a body, and a footer.

Some helpers, like `close`, will be passed down to the slots.
-->
<script lang="ts">
  import Icon from "@iconify/svelte";
  import { createEventDispatcher } from "svelte";

  type Size = "sm" | "lg";

  const dispatch = createEventDispatcher<{ close: undefined }>();

  /**
   * If set the modal won't allow the user to close it (no close button, etc...).
   *
   * _Use with care._
   */
  export let noClose = false;

  export let size: Size = "sm";
</script>

<div class="root" class:sm={size === "sm"} class:lg={size === "lg"}>
  <div class="header">
    <div><slot name="title" /></div>
    {#if !noClose}
      <div class="close" on:click={() => dispatch("close")} title="Close modal">
        <Icon icon="ic:baseline-close" />
      </div>
    {/if}
  </div>
  <div class="body">
    <slot name="body" />
  </div>
  <div class="footer">
    <slot name="footer" />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col bg-gray-700 rounded-sm shadow-lg shadow-black max-h-[calc(100vh-5rem)];
  }

  .root.sm {
    @apply w-96;
  }

  .root.lg {
    @apply w-2/3;
  }

  .header {
    @apply flex flex-row justify-between items-center border-b-2 border-orange-700 px-2 py-1 text-lg h-12 font-semibold;
  }

  .close {
    @apply flex justify-center items-center cursor-pointer text-orange-700;
  }

  .body {
    @apply flex h-full w-full shadow-lg shadow-gray-800 overflow-hidden;
  }

  .footer {
    @apply bg-gray-500 rounded-b-sm px-2 py-1;
  }
</style>
