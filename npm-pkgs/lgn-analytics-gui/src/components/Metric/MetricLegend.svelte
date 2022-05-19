<script lang="ts">
  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricState } from "./Lib/MetricState";
  import type { MetricStore } from "./Lib/MetricStore";

  export let metricStore: MetricStore;
  export let metric: MetricState;

  const iconMap = new Map<string, string>([
    ["us", "bi bi-hourglass-split"],
    ["frame_id", "bi bi-arrow-up-right"],
  ]);

  $: color = metric.hidden ? "rgb(203 213 225)" : getMetricColor(metric.name);

  $: icon = iconMap.get(metric.unit);
</script>

<div
  class="flex align-middle gap-1 select-none cursor-pointer"
  on:click={() => metricStore.switchHidden(metric.name)}
>
  <span class="h-4 w-4 block" style="background-color:{color}" />
  <span class="text-sm flex space-x-1 {metric.hidden ? 'text' : 'black'}">
    <span>{metric.name}</span>
    {#if metric.unit}<span style="color:{color}">({metric.unit})</span>{/if}
  </span>
  {#if icon}
    <i class="{icon} {metric.hidden ? 'text' : 'headline'}" />
  {/if}
</div>
