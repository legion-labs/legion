<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{
    change: string;
    clear: undefined;
  }>();

  export let options: { label: string; value: string }[];

  export let value: string;

  function onChange(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      dispatch("change", event.target.value);
    }
  }
</script>

<div class="search-input">
  <select class="input" on:change={onChange} bind:value>
    {#each options as { label, value }}
      <option {value}>{label}</option>
    {/each}
  </select>
  <div class="clear" on:click={() => dispatch("clear")}>
    <i class="bi-x-circle" />
  </div>
</div>

<style lang="postcss">
  .search-input {
    @apply flex flex-row space-x-0.5;
  }

  .input {
    @apply h-8 px-2 text bg-default outline-none rounded-l-xs;
  }

  .clear {
    @apply flex h-8 w-8 justify-center items-center bg-default text hover:headline cursor-pointer rounded-r-xs;
  }
</style>
