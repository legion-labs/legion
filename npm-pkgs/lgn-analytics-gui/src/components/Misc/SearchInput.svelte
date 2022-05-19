<script lang="ts">
  import { onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";

  import { debounce } from "@lgn/web-client/src/lib/event";

  const dispatch = createEventDispatcher<{
    debouncedInput: { value: string; encodedValue: string };
    clear: undefined;
  }>();

  export let value: string;

  export let placeholder: string | undefined = undefined;

  export let autofocus: boolean | undefined = undefined;

  export let debounceMs = 300;

  const debouncedInput = debounce((event) => {
    if (event.target instanceof HTMLInputElement) {
      dispatch("debouncedInput", {
        value: event.target.value,
        encodedValue: encodeURIComponent(event.target.value),
      });
    }
  }, debounceMs);

  onDestroy(() => {
    debouncedInput.clear();
  });
</script>

<div class="search-input">
  <!-- svelte-ignore a11y-autofocus -->
  <input
    {autofocus}
    type="text"
    class="input"
    {placeholder}
    on:keyup={debouncedInput}
    bind:value
  />
  <div class="clear" on:click={() => dispatch("clear")}>
    <i class="bi-x-circle" />
  </div>
</div>

<style lang="postcss">
  .search-input {
    @apply flex flex-row space-x-0.5;
  }

  .input {
    @apply h-8 w-96 text rounded-l-xs pl-2 bg-default outline-none;
  }

  .clear {
    @apply flex h-8 w-8 justify-center items-center bg-default text hover:headline cursor-pointer rounded-r-xs;
  }
</style>
