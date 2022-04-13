<script lang="ts">
  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricState } from "./Lib/MetricState";
  import type { MetricStore } from "./Lib/MetricStore";

  export let metricStore: MetricStore;
  export let metric: MetricState;
</script>

<div
  on:click={() => metricStore.switchSelection(metric.name)}
  class="select-none"
>
  <input
    type="checkbox"
    style="accent-color:{getMetricColor(metric.name)}"
    checked={metric.selected}
  />
  {metric.name}
  <span style="color:{getMetricColor(metric.name)}">
    ({metric.unit})
  </span>
</div>
