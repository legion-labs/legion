<script lang="ts">
  import type { MetricStore } from "./Lib/MetricStore";
  import MetricLegend from "./MetricLegend.svelte";

  export let metricStore: MetricStore;
</script>

<div class="flex gap-3">
  {#each $metricStore as metric}
    {#if metric.selected}
      <MetricLegend {metricStore} {metric} />
    {/if}
  {/each}
</div>
