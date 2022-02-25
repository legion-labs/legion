<script lang="ts">
  import { getMetricColor } from "@/lib/Metric/MetricColor";
  import { updateMetricSelection } from "@/lib/Metric/MetricSelectionStore";
  import { MetricSelectionState } from "./MetricSelectionState";
  export let metric: MetricSelectionState;
  const iconMap = new Map<string, string>([
    ["us", "bi bi-hourglass-split"],
    ["frame_id", "bi bi-arrow-up-right"],
  ]);
  $: color = metric.hidden ? "rgb(203 213 225)" : getMetricColor(metric.name);
</script>

<div
  class="flex align-middle gap-1 select-none cursor-pointer"
  on:click={() => {
    metric.hidden = !metric.hidden;
    updateMetricSelection(metric);
  }}
>
  <span class="block" style="background-color:{color}" />
  <span class="text-sm {metric.hidden ? 'text-slate-300' : 'black'}"
    >{metric.name} ({metric.unit})</span
  >
  <i
    class="{iconMap.get(metric.unit) ??
      'bi bi-question-circle-fill'} {metric.hidden
      ? 'text-slate-300'
      : 'text-slate-500'}"
  />
</div>

<style>
  .block {
    height: 16px;
    width: 16px;
    display: inline-block;
  }
</style>
