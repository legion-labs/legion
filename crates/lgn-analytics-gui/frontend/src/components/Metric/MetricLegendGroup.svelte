<script lang="ts">
  import { selectionStore } from "@/lib/Metric/MetricSelectionStore";
  import { createEventDispatcher } from "svelte";
  import MetricLegend from "./MetricLegend.svelte";
</script>

<div class="flex gap-3">
  {#each $selectionStore as metric}
    {#if metric.selected}
      <MetricLegend {metric} />
    {/if}
  {/each}
</div>
