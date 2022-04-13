<script lang="ts">
  import { derived, writable } from "svelte/store";

  import clickOutside from "@lgn/web-client/src/actions/clickOutside";

  import MetricSelectionItem from "./MetricSelectionItem.svelte";
  import { getRecentlyUsedStore, type MetricStore } from "./Lib/MetricStore";

  export let metricStore: MetricStore;

  let show = false;
  let searchString = writable<string | null>(null);
  let recentlyUsedMetrics = getRecentlyUsedStore(metricStore);
  let selectedMetricCount = derived(
    metricStore,
    (s) => s.filter((m) => m.selected).length
  );
  let filteredMetrics = derived(
    [metricStore, searchString],
    ([data, search]) => {
      if (!search) {
        return data;
      }
      const lower = search.toLowerCase();
      return data.filter((m) => m.name.toLowerCase().includes(lower));
    }
  );

  function close() {
    show = false;
    $searchString = "";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (show && event.code == "Escape") {
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  use:clickOutside
  on:click-outside={() => {
    close();
  }}
>
  <div
    class="cursor-pointer select-none text-left font-semibold"
    on:click={() => (show = !show)}
  >
    Choose metric <i class="bi bi-caret-{!show ? 'down' : 'up'}-fill" />
  </div>
  {#if show}
    <div class="bg-gray-50 shadow-md border-2 main p-4 absolute z-10">
      <div style="width:850px">
        <div class="flex justify-between pb-4">
          <div>
            <input
              type="text"
              class="metric-search border-2"
              placeholder="Search metric..."
              bind:value={$searchString}
            />
          </div>
          <div class="flex space-x-3 select-none">
            <div class="text-gray-400">
              {$selectedMetricCount} metric{$selectedMetricCount > 1 ? "s" : ""}
              selected
            </div>
            <div on:click={() => metricStore.clearSelection()}>
              <i class="bi bi-x-circle" />
              Clear all
            </div>
          </div>
        </div>
        <div class="grid grid-cols-3">
          <div class=" metric-scrollable ">
            <div class="metric-category-header select-none">Recently Used</div>
            <div class="grid grid-cols-1 justify-items-start">
              {#each $recentlyUsedMetrics as metric}
                <MetricSelectionItem {metricStore} {metric} />
              {/each}
            </div>
          </div>
          <div class="col-span-2 metric-scrollable">
            <div class="metric-category-header select-none">All Metrics</div>
            <div class="grid grid-cols-2 justify-items-start">
              {#each $filteredMetrics as metric}
                <MetricSelectionItem {metricStore} {metric} />
              {/each}
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style lang="postcss">
  .metric-category-header {
    @apply font-semibold text-left;
  }

  .metric-scrollable {
    height: 200px;
    overflow: auto;
  }

  .main {
    width: fit-content;
  }
</style>
