<script lang="ts">
  import clickOutside from "@lgn/web-client/src/actions/clickOutside";
  import MetricSelectionItem from "./MetricSelectionItem.svelte";
  import { MetricSelectionState } from "./MetricSelectionState";
  let show = true;
  let searchString: string | undefined;
  let totalMetrics = Array.from(Array(125).keys()).map(
    (k) => new MetricSelectionState(`Metric With Biscuit ${k}`, "ms")
  );

  function onSearchChange(
    evt: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    searchString = evt.currentTarget.value;
    totalMetrics = totalMetrics;
  }

  $: metrics = totalMetrics.filter((m) => filterMetric(m)).slice(5);
  $: recentMetrics = totalMetrics.slice(0, 5);
  $: selectedMetricCount = totalMetrics.filter((m) => m.selected).length;

  function filterMetric(m: MetricSelectionState) {
    if (!searchString) {
      return true;
    }
    return m.name.toLowerCase().includes(searchString.toLowerCase());
  }

  function onMetricSwitched(e: CustomEvent) {
    if (e.detail.metric instanceof MetricSelectionState) {
      const index = totalMetrics.indexOf(e.detail.metric);
      if (index) {
        totalMetrics[index] = e.detail.metric;
        totalMetrics = totalMetrics;
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (show && event.code == "Escape") {
      show = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  use:clickOutside
  on:click-outside={() => {
    show = false;
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
              class="metric-search border-b-2 border-t-2 border-l-2"
              placeholder="Search metric..."
              on:input={onSearchChange}
            />
            <button class="bg-gray-200 inline-block w-6 h-fit">
              <i class="bi bi-search" />
            </button>
          </div>
          <div class="flex space-x-3 select-none">
            <div class="text-gray-400">
              {selectedMetricCount} metric{selectedMetricCount > 1 ? "s" : ""} selected
            </div>
            <div
              on:click={() => {
                totalMetrics.forEach((m) => (m.selected = false));
                totalMetrics = [...totalMetrics];
                return;
              }}
            >
              <i class="bi bi-x-circle" />
              Clear all
            </div>
          </div>
        </div>
        <div class="grid grid-cols-3">
          <div class=" metric-scrollable ">
            <div class="metric-category-header select-none">Recently Used</div>
            <div class="grid grid-cols-1 justify-items-start">
              {#each recentMetrics as metric}
                <MetricSelectionItem
                  on:metric-switched={(e) => onMetricSwitched(e)}
                  {metric}
                />
              {/each}
            </div>
          </div>
          <div class="col-span-2 metric-scrollable">
            <div class="metric-category-header select-none">All Metrics</div>
            <div class="grid grid-cols-2 justify-items-start">
              {#each metrics as metric}
                <MetricSelectionItem
                  on:metric-switched={(e) => onMetricSwitched(e)}
                  {metric}
                />
              {/each}
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
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

  button:hover {
    @apply bg-gray-300;
  }
</style>
