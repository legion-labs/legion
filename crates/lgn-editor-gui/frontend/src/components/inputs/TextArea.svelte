<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  type Range = readonly [number, number];

  // Type are not preserved when using the `on:input` shortcut
  // so we must use dispatch and explicitely type it
  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  export let value: string;

  export let status: "default" | "error" = "default";

  /** Basically an `width: 100%` style so that the parent can control the width */
  export let fluid = false;

  export let autoFocus = false;

  export let rows = 3;

  export let cols: number | null = 2000;

  /**
   * Auto select the input content on focus,
   * if `true` the whole text is selected,
   * a Range can be provided if you need more control
   */
  export let autoSelect: boolean | Range = false;

  export let disabled = false;

  export let readonly = false;

  export let placeholder: string | null = null;

  let input: HTMLTextAreaElement | undefined;

  $: inactive = disabled || readonly;

  onMount(() => {
    if (autoFocus && input) {
      input.focus();
    }
  });

  function onFocus() {
    if (!input || !autoSelect) {
      return;
    }

    if (typeof autoSelect === "boolean") {
      input.select();
    } else {
      const [from, to] = autoSelect;

      input.setSelectionRange(from, to);
    }
  }

  function onInput(
    event: Event & {
      currentTarget: EventTarget & HTMLTextAreaElement;
    }
  ) {
    dispatch("input", event.currentTarget.value);
  }
</script>

<div
  class="root"
  class:w-full={fluid}
  class:disabled
  class:readonly
  class:error={status === "error" && !inactive}
>
  <textarea
    class="input"
    class:disabled
    autocomplete="none"
    aria-autocomplete="none"
    type="text"
    {disabled}
    {readonly}
    {placeholder}
    {rows}
    {cols}
    bind:value
    bind:this={input}
    on:input={inactive ? null : onInput}
    on:focus={inactive ? null : onFocus}
  />
</div>

<style lang="postcss">
  .root {
    @apply rounded-sm;
  }

  .root.disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .root.readonly {
    @apply text-gray-400;
  }

  .root.error {
    @apply border border-red-700;
  }

  .input {
    @apply bg-gray-800 border-gray-400 px-2 rounded-sm outline-none w-full h-full resize-none;
  }

  .input.disabled {
    @apply cursor-not-allowed;
  }
</style>
