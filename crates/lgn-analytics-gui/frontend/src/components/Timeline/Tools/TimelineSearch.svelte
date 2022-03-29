<script lang="ts">
  import { TimelineContext } from "../Stores/TimelineContext";
  const searchStore = TimelineContext.search;
</script>

<div>
  <input
    type="text"
    class="metric-search border-2"
    placeholder="Search..."
    bind:value={$searchStore}
  />
  <button on:click={() => searchStore.set("")}
    ><i class="bi bi-x-circle" /></button
  >
</div>

<style lang="postcss">
  input {
    @apply text-sm;
    border: 0px;
  }

  button {
    @apply text-sm bg-slate-300 text-slate-50 w-5;
  }
</style>
