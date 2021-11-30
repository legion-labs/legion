<script lang="ts">
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
    on:input
    on:focus={onFocus}
    {value}
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
