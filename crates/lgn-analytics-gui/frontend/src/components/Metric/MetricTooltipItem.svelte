<script lang="ts">
  import { getMetricColor } from "@/lib/Metric/MetricColor";
  import { Point } from "@/lib/Metric/MetricPoint";
  import { MetricSelectionState } from "./MetricSelectionState";
  export let metric: MetricSelectionState;
  export let value: Point | null;
</script>

<div class="flex align-middle justify-between gap-1 select-none">
  <div class="flex align-middle gap-1 ">
    <span
      class="block"
      style="background-color:{getMetricColor(metric.name)}"
    />
    <span class="name">{metric.name}</span>
  </div>
  <div class="pl-2">
    <span class="font-semibold">{value?.value.toLocaleString()}</span>
    <span class="pl-1 text-slate-500">({metric.unit})</span>
  </div>
</div>

<style>
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .block {
    height: 16px;
    width: 16px;
    display: inline-block;
  }
</style>
