<script lang="ts">
  import {
    getRecentlyUsedMetrics,
    selectionStore,
    updateMetricSelection,
  } from "@/lib/Metric/MetricSelectionStore";
  import clickOutside from "@lgn/web-client/src/actions/clickOutside";
  import { onDestroy, onMount } from "svelte";
  import { get, Unsubscriber } from "svelte/store";
  import MetricSelectionItem from "./MetricSelectionItem.svelte";
  import { MetricSelectionState } from "./MetricSelectionState";

  let show = false;
  let searchString: string | undefined;
  let subscription: Unsubscriber;
  let selectedMetricCount: number;
  let filteredMetrics: MetricSelectionState[];
  let recentlyUsedMetrics: MetricSelectionState[];

  onMount(() => {
    subscription = selectionStore.subscribe((selections) => {
      selectedMetricCount = selections.filter((m) => m.selected).length;
      filteredMetrics = getFilteredMetrics(selections);
      recentlyUsedMetrics = selections.filter((m) => recentlyUsedFilter(m));
    });
  });

  onDestroy(() => {
    if (subscription) {
      subscription();
    }
  });

  function getFilteredMetrics(selection: MetricSelectionState[]) {
    return selection.filter((m) => filterMetric(m));
  }

  function onSearchChange(
    e: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    updateSearch(e.currentTarget.value);
  }

  function updateSearch(value: string) {
    searchString = value;
    filteredMetrics = getFilteredMetrics(get(selectionStore));
  }

  function close() {
    show = false;
    updateSearch("");
  }

  function filterMetric(m: MetricSelectionState) {
    if (!searchString) {
      return true;
    }
    return m.name.toLowerCase().includes(searchString.toLowerCase());
  }

  function recentlyUsedFilter(metric: MetricSelectionState) {
    const recent = getRecentlyUsedMetrics();
    return recent.some((m) => m.name === metric.name);
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
              on:input={onSearchChange}
            />
          </div>
          <div class="flex space-x-3 select-none">
            <div class="text-gray-400">
              {selectedMetricCount} metric{selectedMetricCount > 1 ? "s" : ""} selected
            </div>
            <div
              on:click={() => {
                get(selectionStore).forEach((m) => {
                  m.selected = false;
                  updateMetricSelection(m);
                });
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
              {#each recentlyUsedMetrics as metric}
                <MetricSelectionItem {metric} />
              {/each}
            </div>
          </div>
          <div class="col-span-2 metric-scrollable">
            <div class="metric-category-header select-none">All Metrics</div>
            <div class="grid grid-cols-2 justify-items-start">
              {#each filteredMetrics as metric}
                <MetricSelectionItem {metric} />
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
</style>
