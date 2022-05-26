<script lang="ts">
  import { getContext } from "svelte";

  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricState } from "./Lib/MetricState";
  import type { MetricStore } from "./Lib/MetricStore";

  export let metric: MetricState;

  const metricStore = getContext<MetricStore>("metrics-store");

  $: color = getMetricColor(metric.name);
</script>

<div
  on:click={() => metricStore.switchSelection(metric.name)}
  class="metric-selection-item"
>
  <div class="pt-0.5">
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

<style lang="postcss">
  .metric-selection-item {
    @apply flex space-x-1 break-all select-none w-full border-b border-[#3d3d3d] border-dotted cursor-pointer;
  }
</style>
