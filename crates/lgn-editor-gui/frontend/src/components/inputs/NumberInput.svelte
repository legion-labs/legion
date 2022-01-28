<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  type Size = "default";

  // Type are not preserved when using the `on:input` shortcut
  // so we must use dispatch and explicitely type it
  const dispatch = createEventDispatcher<{
    input: number;
  }>();

  export let value: number;

  export let min: number | undefined = undefined;

  export let max: number | undefined = undefined;

  export let size: Size = "default";

  export let fullWidth = false;

  export let noArrow = false;

  export let autoFocus = false;

  export let autoSelect = false;

  export let step = 0;

  export let align: "right" | "left" = "left";

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
    // Svelte will not call this function if the input value
    // is not a valid number, so we can safely cast it to `number`
    dispatch("input", +event.currentTarget.value);
  }
</script>

<input
  class="input"
  class:default={size === "default"}
  class:disabled
  class:w-full={fullWidth}
  class:no-arrow={noArrow}
  class:text-right={align === "right"}
  autocomplete="none"
  aria-autocomplete="none"
  type="number"
  {min}
  {max}
  {step}
  {disabled}
  bind:this={input}
  bind:value
  on:input={disabled ? null : onInput}
  on:focus={onFocus}
/>

<style lang="postcss">
  .input {
    @apply bg-gray-800 border-gray-400 px-2 py-1 rounded-sm outline-none;
  }

  .disabled {
    @apply text-gray-400 cursor-not-allowed;
  }

  .default {
    @apply h-8;
  }
</style>
