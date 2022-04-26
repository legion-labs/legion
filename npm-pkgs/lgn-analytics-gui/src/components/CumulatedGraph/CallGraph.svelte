<script lang="ts">
  import { onMount } from "svelte";
  import { BarLoader } from "svelte-loading-spinners";
  import { link } from "svelte-navigator";

  import { endQueryParam, startQueryParam } from "@/lib/time";

  import CallTreeDebug from "./CallGraphDebug.svelte";
  import CallGraphLine from "./CallGraphLine.svelte";
  import { getProcessCumulatedCallGraph } from "./Lib/CallGraphStore";
  import type { CumulatedCallGraphStore } from "./Lib/CallGraphStore";

  export let begin: number;
  export let end: number;
  export let processId: string;
  export let debug = false;
  export let size: number;

  let store: CumulatedCallGraphStore;
  let tickTimer: number;

  $: (begin || end) && tick();

  function tick() {
    clearTimeout(tickTimer);
    tickTimer = setTimeout(async () => {
      await store?.updateRange(begin, end);
    }, 500);
  }

  onMount(async () => {
    store = await getProcessCumulatedCallGraph(processId, begin, end);
  });
</script>

{#if store && $store.loading}
  <div class="flex items-center justify-center h-full">
    <BarLoader size={32} />
  </div>
{:else if store}
  {#if debug}
    <CallTreeDebug {store} {begin} {end} />
  {/if}
  <div class="overflow-auto" style:max-height={`${size}px`}>
    <table class="w-full bg-skin-700 text-xs text-content-60 space-y-2 ">
      <tr class="bg-skin-800 w-100">
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
            <CallGraphLine {node} {store} threadId={key} />
          {/each}
        {/if}
      {/each}
    </table>
  </div>
  <div
    class="text-content-38 bg-content-38  hover:bg-content-60 p-1 mt-1 text-xs float-right "
  >
    <a
      href={`/cumulative-call-graph?process=${processId}&${startQueryParam}=${begin}&${endQueryParam}=${end}`}
      target="_blank"
      use:link
    >
      <i class="bi bi-arrow-up-right-circle" />
      Open Cumulative Call Graph
    </a>
  </div>
{/if}

<style lang="postcss">
  i {
    @apply pr-1;
  }

  ::-webkit-scrollbar {
    width: 20px;
  }

  ::-webkit-scrollbar-track {
    background-color: transparent;
  }

  ::-webkit-scrollbar-corner {
    background: rgba(0, 0, 0, 0);
  }

  ::-webkit-scrollbar-thumb {
    background-color: #c4baba;
    border-radius: 20px;
    border: 6px solid transparent;
    background-clip: content-box;
  }

  ::-webkit-scrollbar-thumb:hover {
    background-color: #9e9696;
  }
</style>
