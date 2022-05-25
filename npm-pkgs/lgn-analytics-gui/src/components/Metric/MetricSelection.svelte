<script lang="ts">
  import { derived, writable } from "svelte/store";

  import clickOutside from "@lgn/web-client/src/actions/clickOutside";

  import { getL10nOrchestratorContext } from "@/contexts";

  import L10n from "../Misc/L10n.svelte";
  import { getRecentlyUsedStore } from "./Lib/MetricStore";
  import type { MetricStore } from "./Lib/MetricStore";
  import MetricSelectionItem from "./MetricSelectionItem.svelte";

  const { t } = getL10nOrchestratorContext();

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
    <L10n id="metrics-search-choose-metrics" />
    <i class="bi bi-caret-{!show ? 'down' : 'up'}-fill" />
  </div>
  {#if show}
    <div class="main">
      <div class="flex justify-between pb-4 flex-shrink-0">
        <div>
          <input
            type="text"
            class="h-8 w-96 placeholder rounded-sm pl-2 surface"
            placeholder={$t("metrics-search-placeholder")}
            bind:value={$searchString}
          />
        </div>
        <div class="flex space-x-3 select-none">
          <div class="text-gray-400">
            <L10n
              id="metrics-search-result-number"
              variables={{ selectedMetricCount: $selectedMetricCount }}
            />
          </div>
          <div on:click={() => metricStore.clearSelection()}>
            <i class="bi bi-x-circle" />
            <L10n id="metrics-search-clear" />
          </div>
        </div>
      </div>
      <div class="metrics">
        <div class="metric w-1/3">
          <div class="metric-category-header">
            <L10n id="metrics-recently-used" />
          </div>
          <div class="metric-list">
            {#each $recentlyUsedMetrics as metric}
              <MetricSelectionItem {metricStore} {metric} />
            {/each}
          </div>
        </div>
        <div class="metric w-2/3">
          <div class="metric-category-header">
            <L10n id="metrics-all-metrics" />
          </div>
          <div class="metric-list">
            <div class="columns">
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
  .main {
    @apply on-surface shadow-xl px-2 py-4 absolute z-10 h-80 w-[60rem] flex flex-col;
  }

  .metrics {
    @apply flex flex-row space-x-4 flex-grow overflow-hidden;
  }

  .metric {
    @apply flex flex-col;
  }

  .metric-category-header {
    @apply font-semibold text-left select-none h-10;
  }

  .metric-list {
    @apply overflow-y-auto;
  }

  .columns {
    @apply columns-2 gap-4;
  }
</style>
