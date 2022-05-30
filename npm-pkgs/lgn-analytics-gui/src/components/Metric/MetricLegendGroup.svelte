<script lang="ts">
  import { getContext } from "svelte";

  import MetricLegend from "./MetricLegend.svelte";

  const selectedMetricStore = getContext("selected-metrics-store");
</script>

<div class="flex gap-3 flex-wrap">
  {#each $selectedMetricStore as metric, index}
    <MetricLegend {metric} {index} />
  {/each}
</div>
