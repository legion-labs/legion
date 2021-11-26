<script lang="ts">
  export let value: string;

  export let size: "sm" | "default" = "default";

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
  class:sm={size === "sm"}
  class:default={size === "default"}
  class:root-with-extension={$$slots.extension}
>
  <input
    class="input"
    class:input-with-extension={$$slots.extension}
    type="text"
    on:focus={onFocus}
    bind:value
    bind:this={input}
  />
  {#if $$slots.extension}
    <div
      class="extension"
      class:extension-sm={size === "sm"}
      class:extension-default={size === "default"}
    >
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

  .sm {
    @apply h-7 text-sm;
  }

  .default {
    @apply h-8;
  }

  .extension {
    @apply border-l rounded-r-sm bg-gray-800 border-gray-700 h-full p-1;
  }

  .extension-sm {
    @apply w-7;
  }

  .extension-default {
    @apply w-8;
  }
</style>
