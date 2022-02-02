<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  // Type are not preserved when using the `on:input` shortcut
  // so we must use dispatch and explicitely type it
  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  export let value: string;

  export let status: "default" | "error" = "default";

  export let size: "default" | "sm" | "lg" = "default";

  export let fullWidth = false;

  export let autoFocus = false;

  export let autoSelect = false;

  export let disabled = false;

  export let placeholder: string | null = null;

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
  class="root"
  class:w-full={fullWidth}
  class:disabled
  class:default={size === "default"}
  class:sm={size === "sm"}
  class:lg={size === "lg"}
  class:error={status === "error" && !disabled}
  class:with-extension={$$slots.rightExtension || $$slots.leftExtension}
>
  {#if $$slots.leftExtension}
    <div
      class="extension left-extension"
      class:extension-default={size === "default"}
      class:extension-sm={size === "sm"}
      class:extension-lg={size === "lg"}
    >
      <slot name="leftExtension" />
    </div>
  {/if}
  <input
    class="input"
    class:disabled
    class:with-right-extension={$$slots.rightExtension}
    class:with-left-extension={$$slots.leftExtension}
    class:default={size === "default"}
    class:sm={size === "sm"}
    class:lg={size === "lg"}
    autocomplete="none"
    aria-autocomplete="none"
    type="text"
    {disabled}
    {placeholder}
    bind:value
    bind:this={input}
    on:input={disabled ? null : onInput}
    on:focus={disabled ? null : onFocus}
  />
  {#if $$slots.rightExtension}
    <div
      class="extension right-extension"
      class:extension-default={size === "default"}
      class:extension-sm={size === "sm"}
      class:extension-lg={size === "lg"}
    >
      <slot name="rightExtension" />
    </div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply rounded-sm;
  }

  .root.with-extension {
    @apply flex flex-row;
  }

  .root.disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .root.default {
    @apply h-8;
  }

  .root.sm {
    @apply h-6 text-sm;
  }

  .root.lg {
    @apply h-10 text-sm;
  }

  .root.error {
    @apply border border-red-700;
  }

  .input {
    @apply bg-gray-800 border-gray-400 px-2 rounded-sm outline-none w-full h-full;
  }

  .input.disabled {
    @apply cursor-not-allowed;
  }

  .input.default {
    @apply py-1;
  }

  .input.sm {
    @apply py-0.5;
  }

  .input.lg {
    @apply py-1 text-base;
  }

  .input.with-right-extension {
    @apply rounded-r-none;
  }

  .input.with-left-extension {
    @apply rounded-l-none;
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

  .extension-sm {
    @apply w-6 flex-shrink-0;
  }

  .extension-lg {
    @apply w-10 flex-shrink-0;
  }
</style>
