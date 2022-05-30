<script lang="ts">
  import { getContext } from "svelte";

  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricState } from "./Lib/MetricState";

  export let metric: MetricState;
  export let index: number;

  const metricStore = getContext("metrics-store");

  const iconMap = new Map<string, string>([
    ["us", "bi bi-hourglass-split"],
    ["frame_id", "bi bi-arrow-up-right"],
  ]);

  $: color = getMetricColor(index);

  $: icon = iconMap.get(metric.unit);
</script>

<div
  class="flex align-middle gap-1 select-none cursor-pointer"
  on:click={() => metricStore.switchSelection(metric.name)}
>
  <span class="h-4 w-4 block" style="background-color:{color}" />
  <span class="text-sm flex space-x-1 black">
    <span>{metric.name}</span>
    {#if metric.unit}<span style="color:{color}">({metric.unit})</span>{/if}
  </span>
  {#if icon}
    <i class="{icon} headline" />
  {/if}
</div>
