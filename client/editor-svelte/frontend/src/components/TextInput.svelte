<script lang="ts">
  import { createEventDispatcher } from "svelte";

  // Type are not preserved when using the `on:input` shortcut
  // so we must use dispatch and explicitely type it
  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  export let value: string;

  export let size: "default" = "default";

  export let fullWidth = false;

  export let autoSelect = false;

  let input: HTMLInputElement | undefined;

  const onFocus = (_event: FocusEvent) => {
    if (autoSelect && input) {
      input.select();
    }
  };

  const onInput = (
    event: Event & {
      currentTarget: EventTarget & HTMLInputElement;
    }
  ) => {
    dispatch("input", event.currentTarget.value);
  };
</script>

<div
  class:w-full={fullWidth}
  class:default={size === "default"}
  class:root-with-extension={$$slots.extension}
>
  <input
    class="input"
    class:input-with-extension={$$slots.extension}
    type="text"
    on:input={onInput}
    on:focus={onFocus}
    bind:value
    bind:this={input}
  />
  {#if $$slots.extension}
    <div class="extension" class:extension-default={size === "default"}>
      <slot name="extension" />
    </div>
  {/if}
</div>

<style lang="postcss">
  .root-with-extension {
    @apply flex flex-row;
  }

  .input {
    @apply bg-gray-800 border-gray-400 px-2 py-1 rounded-sm outline-none w-full;
  }

  .input-with-extension {
    @apply rounded-r-none;
  }

  .default {
    @apply h-8;
  }

  .extension {
    @apply border-l rounded-r-sm bg-gray-800 border-gray-700 h-full p-1;
  }

  .extension-default {
    @apply w-8;
  }
</style>
