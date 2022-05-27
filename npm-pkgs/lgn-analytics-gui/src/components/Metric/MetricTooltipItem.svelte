<script lang="ts">
  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricPoint } from "./Lib/MetricPoint";
  import type { MetricState } from "./Lib/MetricState";

  export let metric: MetricState;
  export let value: MetricPoint | null;

  $: color = getMetricColor(metric.name);
</script>

<div class="flex align-middle justify-between gap-1 select-none">
  <div class="flex align-middle gap-1 ">
    <span class="w-4 h-4 inline-block" style="background-color:{color}" />
    <span class="overflow-hidden text-ellipsis">{metric.name}</span>
  </div>
  <div class="pl-2">
    <span class="font-semibold">{value?.value.toLocaleString()}</span>
    {#if metric.unit}
      <span class="pl-1" style="color: {color}">({metric.unit})</span>
    {/if}
  </div>
</div>
