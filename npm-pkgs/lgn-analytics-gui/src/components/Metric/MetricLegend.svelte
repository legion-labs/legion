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
</script>

<div
  class="flex align-middle gap-1 select-none cursor-pointer"
  on:click={() => metricStore.switchHidden(metric.name)}
>
  <span class="h-4 w-4 block" style="background-color:{color}" />
  <span class="text-sm {metric.hidden ? 'text-text' : 'black'}">
    {metric.name} ({metric.unit})</span
  >
  <i
    class="{iconMap.get(metric.unit) ??
      'bi bi-question-circle-fill'} {metric.hidden
      ? 'text-text'
      : 'text-headline'}"
  />
</div>
