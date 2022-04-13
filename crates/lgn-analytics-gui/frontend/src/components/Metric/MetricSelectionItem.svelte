<script lang="ts">
  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricSelectionState } from "./Lib/MetricSelectionState";
  import { updateMetricSelection } from "./Lib/MetricSelectionStore";

  export let metric: MetricSelectionState;

  function switchMetric() {
    metric.selected = !metric.selected;
    if (metric.selected) {
      metric.hidden = false;
    }
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
