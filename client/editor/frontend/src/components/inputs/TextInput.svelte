<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  type Size = "default";

  // Type are not preserved when using the `on:input` shortcut
  // so we must use dispatch and explicitely type it
  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  export let value: string;

  export let size: Size = "default";

  export let fullWidth = false;

  export let autoFocus = false;

  export let autoSelect = false;

  export let disabled = false;

  let input: HTMLInputElement | undefined;

  onMount(() => {
    if (autoFocus && input) {
      input.focus();
    }
  });

  function onFocus() {
    if (autoSelect && input) {
      input.select();
    }
  }

  function onInput(
    event: Event & {
      currentTarget: EventTarget & HTMLInputElement;
    }
  ) {
    dispatch("input", event.currentTarget.value);
  }
</script>

<div
  class:w-full={fullWidth}
  class:disabled
  class:default={size === "default"}
  class:root-with-extension={$$slots.rightExtension || $$slots.leftExtension}
>
  {#if $$slots.leftExtension}
    <div
      class="extension left-extension"
      class:extension-default={size === "default"}
    >
      <slot name="leftExtension" />
    </div>
  {/if}
  <input
    class="input"
    class:disabled
    class:with-right-extension={$$slots.rightExtension}
    class:with-left-extension={$$slots.leftExtension}
    type="text"
    on:input={disabled ? null : onInput}
    on:focus={disabled ? null : onFocus}
    bind:value
    bind:this={input}
    {disabled}
  />
  {#if $$slots.rightExtension}
    <div
      class="extension right-extension"
      class:extension-default={size === "default"}
    >
      <slot name="rightExtension" />
    </div>
  {/if}
</div>

<style lang="postcss">
  .root-with-extension {
    @apply flex flex-row;
  }

  .disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .input {
    @apply bg-gray-800 border-gray-400 px-2 py-1 rounded-sm outline-none w-full;
  }

  .input.disabled {
    @apply cursor-not-allowed;
  }

  .input.with-right-extension {
    @apply rounded-r-none;
  }

  .input.with-left-extension {
    @apply rounded-l-none;
  }

  .default {
    @apply h-8;
  }

  .extension {
    @apply bg-gray-800 border-gray-700 h-full p-1;
  }

  .left-extension {
    @apply border-r rounded-l-sm;
  }

  .right-extension {
    @apply border-l rounded-r-sm;
  }

  .extension-default {
    @apply w-8 flex-shrink-0;
  }

  .extension-default {
    @apply w-8 flex-shrink-0;
  }
</style>
