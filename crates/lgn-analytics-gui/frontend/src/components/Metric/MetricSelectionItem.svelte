<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { MetricSelectionState } from "./MetricSelectionState";
  const dispatcher = createEventDispatcher();
  export let metric: MetricSelectionState;

  function switchMetric() {
    metric.selected = !metric.selected;
    dispatcher("metric-switched", {
      metric,
    });
  }
</script>

<div on:click={switchMetric} class="select-none">
  <input type="checkbox" class="checkbox" checked={metric.selected} />
  {metric.name}
  <span class="text-orange-700">
    ({metric.unit})
  </span>
</div>

<style>
  .checkbox {
    accent-color: #fc4d0f;
  }
</style>
