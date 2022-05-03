<script lang="ts">
  import { onMount } from "svelte";
  import { BarLoader } from "svelte-loading-spinners";
  import { link } from "svelte-navigator";

  import type { CumulatedCallGraphHierarchyStore } from "@/lib/CallGraph/CallGraphStore";
  import { getProcessCumulatedCallGraphHierarchy } from "@/lib/CallGraph/CallGraphStore";
  import { endQueryParam, startQueryParam } from "@/lib/time";

  import CallTreeDebug from "./CallGraphHierachyDebug.svelte";
  import CallGraphLine from "./CallGraphHierachyLine.svelte";

  export let begin: number;
  export let end: number;
  export let processId: string;
  export let size: number;
  export let debug = false;

  let store: CumulatedCallGraphHierarchyStore;

  $: (begin || end) && tick();

  async function tick() {
    await store?.updateRange(begin, end);
  }

  onMount(async () => {
    store = await getProcessCumulatedCallGraphHierarchy(processId, begin, end);
  });
</script>

{#if store}
  {#if $store.loading}
    <div
      class="flex items-center justify-center h-full"
      style:max-height={`${size}px`}
    >
      <BarLoader size={32} />
    </div>
  {:else}
    <div>
      {#if debug}
        <CallTreeDebug {store} {begin} {end} />
      {/if}
      <div class="flex flex-col gap-y-2">
        <div class="surface hover:background px-2 py-1 mt-2 text-xs w-fit">
          <a
            class="placeholder"
            href={`/cumulative-call-graph?process=${processId}&${startQueryParam}=${begin}&${endQueryParam}=${end}`}
            target="_blank"
            use:link
          >
            <i class="bi bi-arrow-up-right-circle" />
            Open Cumulative Call Graph
          </a>
        </div>
        <div class="overflow-auto" style:max-height={`${size}px`}>
          <table class="w-full background text-xs text space-y-2 table-fixed">
            <tr class="background w-100">
              <th style="width:66%" class="text-left">Function</th>
              <th class="table-header"><i class="bi bi-caret-right" />Count</th>
              <th class="table-header"
                ><i class="bi bi-chevron-bar-contract" />Avg</th
              >
              <th class="table-header"
                ><i class="bi bi-chevron-bar-left" />Min</th
              >
              <th class="table-header"
                ><i class="bi bi-chevron-bar-right" />Max</th
              >
              <th class="table-header"><i class="bi bi-lightbulb" />Sd</th>
              <th class="table-header"
                ><i class="bi bi bi-caret-right-fill" /> Sum</th
              >
            </tr>
            {#each Array.from($store.threads) as [hash, thread] (hash)}
              {#if thread.data}
                {#each Array.from(thread.data).filter((obj) => obj[1].parents.size === 0) as [key, node] (key)}
                  <CallGraphLine {node} {store} threadId={key} />
                {/each}
              {/if}
            {/each}
          </table>
        </div>
      </div>
    </div>
  {/if}
{/if}

<style lang="postcss">
  i {
    @apply pr-1;
  }

  .table-header {
    @apply truncate;
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
    background-color: #454545;
    border-radius: 20px;
    border: 6px solid transparent;
    background-clip: content-box;
  }

  ::-webkit-scrollbar-thumb:hover {
    background-color: #707070;
  }
</style>
