<script lang="ts">
  import { getMetricColor } from "@/lib/Metric/MetricColor";
  import { updateMetricSelection } from "@/lib/Metric/MetricSelectionStore";
  import { MetricSelectionState } from "./MetricSelectionState";
  export let metric: MetricSelectionState;

  function switchMetric() {
    metric.selected = !metric.selected;
    updateMetricSelection(metric);
  }
</script>

<div on:click={switchMetric} class="select-none">
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
