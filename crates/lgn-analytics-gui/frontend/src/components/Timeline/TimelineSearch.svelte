<script lang="ts">
  import { TimelineContext } from "@/lib/Timeline/TimelineContext";
  const store = TimelineContext.search;
</script>

<div>
  <input
    type="text"
    class="metric-search border-2"
    placeholder="Search..."
    bind:value={$store}
  />
  <button on:click={() => store.set("")}><i class="bi bi-x-circle" /></button>
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
