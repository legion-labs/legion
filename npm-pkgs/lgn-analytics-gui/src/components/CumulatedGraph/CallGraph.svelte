<script lang="ts">
  import { onMount } from "svelte";
  import { BarLoader } from "svelte-loading-spinners";
  import { getProcessCumulatedCallGraph } from "./Lib/CallGraphStore";
  import type { CumulatedCallGraphStore } from "./Lib/CallGraphStore";
  import CallLine from "./CallLine.svelte";
  import CallTreeDebug from "./CallGraphDebug.svelte";

  export let begin: number;
  export let end: number;
  export let processId: string;
  export let debug = false;

  let loading = true;
  let store: CumulatedCallGraphStore;
  let tickTimer: number;

  $: (begin || end) && tick();

  function tick() {
    loading = true;
    clearTimeout(tickTimer);
    tickTimer = setTimeout(() => {
      store?.updateRange(begin, end).finally(() => (loading = false));
    }, 500);
  }

  onMount(async () => {
    store = await getProcessCumulatedCallGraph(processId, begin, end).finally(
      () => (loading = false)
    );
  });
</script>

{#if loading}
  <div class="flex items-center justify-center h-full">
    <BarLoader size={32} />
  </div>
{:else if store}
  <table class="w-full bg-skin-700 text-xs text-content-60 space-y-2">
    <tr class="bg-skin-800">
      <th class="text-left">Function</th>
      <th><i class="bi bi-caret-right" />Count</th>
      <th><i class="bi bi-chevron-bar-contract" />Avg</th>
      <th><i class="bi bi-chevron-bar-left" />Min</th>
      <th><i class="bi bi-chevron-bar-right" />Max</th>
      <th><i class="bi bi-lightbulb" />Sd</th>
      <th><i class="bi bi bi-caret-right-fill" /> Sum</th>
    </tr>
    {#each Array.from($store.threads) as [hash, thread] (hash)}
      {#if thread.data}
        {#each Array.from(thread.data).filter((obj) => obj[1].parent.size === 0) as [key, node] (key)}
          <CallLine {node} {store} threadId={key} />
        {/each}
      {/if}
    {/each}
  </table>
  {#if debug}
    <CallTreeDebug {store} {begin} {end} />
  {/if}
{/if}

<style lang="postcss">
  i {
    @apply pr-1;
  }
</style>
