<script lang="ts">
  import { selectionStore } from "./Lib/MetricSelectionStore";
  import MetricLegend from "./MetricLegend.svelte";
</script>

<div class="flex gap-3">
  {#each $selectionStore as metric}
    {#if metric.selected}
      <MetricLegend {metric} />
    {/if}
  {/each}
</div>
