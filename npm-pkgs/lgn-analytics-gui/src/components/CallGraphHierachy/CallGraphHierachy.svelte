<script lang="ts">
  import { getContext, onMount } from "svelte";
  import { BarLoader } from "svelte-loading-spinners";

  import type { CumulatedCallGraphHierarchyStore } from "@/lib/CallGraph/CallGraphStore";
  import { getProcessCumulatedCallGraphHierarchy } from "@/lib/CallGraph/CallGraphStore";
  import { endQueryParam, startQueryParam } from "@/lib/time";

  import L10n from "../Misc/L10n.svelte";
  import CallTreeDebug from "./CallGraphHierachyDebug.svelte";
  import CallGraphLine from "./CallGraphHierachyLine.svelte";

  const client = getContext("http-client");

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
    store = await getProcessCumulatedCallGraphHierarchy(
      client,
      processId,
      begin,
      end
    );
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
        <div class="surface hover:background text-xs w-fit">
          <a
            class="flex placeholder h-full w-full px-2 py-1 space-x-1"
            href={`/cumulative-call-graph?process=${processId}&${startQueryParam}=${begin}&${endQueryParam}=${end}`}
            target="_blank"
          >
            <div><i class="bi bi-arrow-up-right-circle" /></div>
            <div><L10n id="timeline-open-cumulative-call-graph" /></div>
          </a>
        </div>
        <div class="overflow-auto" style:max-height={`${size}px`}>
          <div role="table" class="w-full background text-xs text">
            <div
              role="rowgroup"
              class="flex flex-row items-center h-6 w-full background sticky top-0"
            >
              <div role="row" class="flex flex-row background w-full px-1">
                <div role="columnheader" class="text-left w-full">
                  <div><L10n id="timeline-table-function" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi-caret-right" /></div>
                  <div><L10n id="timeline-table-count" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi-chevron-bar-contract" /></div>
                  <div><L10n id="timeline-table-average" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi-chevron-bar-left" /></div>
                  <div><L10n id="timeline-table-minimum" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi-chevron-bar-right" /></div>
                  <div><L10n id="timeline-table-maximum" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi-lightbulb" /></div>
                  <div><L10n id="timeline-table-standard-deviation" /></div>
                </div>
                <div role="columnheader" class="table-header">
                  <div><i class="bi bi bi-caret-right-fill" /></div>
                  <div><L10n id="timeline-table-sum" /></div>
                </div>
              </div>
            </div>
            <div role="rowgroup">
              {#each Array.from($store.threads) as [hash, thread] (hash)}
                {#if thread.data}
                  {#each Array.from(thread.data).filter((obj) => obj[1].parents.size === 0) as [key, node] (key)}
                    <CallGraphLine {node} {store} threadId={key} />
                  {/each}
                {/if}
              {/each}
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
{/if}

<style lang="postcss">
  .table-header {
    @apply flex flex-shrink-0 w-28 truncate space-x-1 justify-center;
  }
</style>
