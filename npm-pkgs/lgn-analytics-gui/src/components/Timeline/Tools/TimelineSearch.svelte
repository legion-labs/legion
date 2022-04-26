<script lang="ts">
  import { TimelineContext } from "../Stores/TimelineContext";

  export let searching: boolean;
  const searchStore = TimelineContext.search;
</script>

<div>
  <input
    type="text"
    class="metric-search border-0 text-sm text-content-60 bg-background"
    placeholder="Search..."
    on:focus={() => (searching = true)}
    on:blur={() => (searching = false)}
    bind:value={$searchStore}
  />
  <button
    class="text-sm bg-background text-content-60 w-5"
    on:click={() => searchStore.set("")}><i class="bi bi-x-circle" /></button
  >
</div>
