<script lang="ts">
  import { TimelineContext } from "../Stores/TimelineContext";

  export let searching: boolean;
  const searchStore = TimelineContext.search;
</script>

<div class="flex flex-row h-full w-full justify-end space-x-1">
  <input
    type="text"
    class="h-8 w-60 placeholder rounded-l-sm pl-2 surface"
    placeholder="Search..."
    on:focus={() => (searching = true)}
    on:blur={() => (searching = false)}
    bind:value={$searchStore}
  />
  <button
    class="text-sm surface h-8 w-8 rounded-r-sm text-opacity-60"
    on:click={() => searchStore.set("")}
  >
    <i class="bi bi-x-circle" />
  </button>
</div>
