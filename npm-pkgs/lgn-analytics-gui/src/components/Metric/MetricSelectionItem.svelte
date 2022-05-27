<script lang="ts">
  import { getContext } from "svelte";

  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricState } from "./Lib/MetricState";

  export let metric: MetricState;

  const metricStore = getContext("metrics-store");

  $: color = getMetricColor(metric.name);
</script>

<div
  on:click={() => metricStore.switchSelection(metric.name)}
  class="metric-selection-item-wrapper"
>
  <div class="metric-selection-item">
    <div>
      <input
        type="checkbox"
        style="accent-color:{color}"
        checked={metric.selected}
      />
    </div>
    <div>
      {metric.name}
      {#if metric.unit}
        <span style="color:{color}">
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
