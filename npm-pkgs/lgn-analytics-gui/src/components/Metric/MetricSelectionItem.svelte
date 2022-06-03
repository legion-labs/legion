<script lang="ts">
  import { getContext } from "svelte";

  import type { MetricState } from "./Lib/MetricState";

  export let metric: MetricState;

  const metricStore = getContext("metrics-store");
</script>

<div
  on:click={() => metricStore.switchSelection(metric.name)}
  class="metric-selection-item-wrapper"
>
  <div class="metric-selection-item">
    <div>
      <input type="checkbox" checked={metric.selected} />
    </div>
    <div>
      {metric.name}
      {#if metric.unit}
        <span>
          ({metric.unit})
        </span>
      {/if}
    </div>
  </div>
</div>

<style lang="postcss">
  .metric-selection-item-wrapper {
    @apply select-none w-full border-b border-[#3d3d3d] py-0.5 border-dotted break-inside-avoid-column;
  }

  .metric-selection-item {
    @apply flex space-x-1 break-all w-full cursor-pointer;
  }
</style>
