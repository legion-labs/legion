<script lang="ts">
  import type { NodeStats } from "@lgn/proto-telemetry/dist/analytics";

  import GraphNodeStatItem from "./GraphNodeStatItem.svelte";
  import { GraphNodeStatType } from "./Lib/GraphNodeStatType";

  export let data: NodeStats;
</script>

<div class="flex flex-row justify-between select-none h-8 text-content-87">
  <div class="items-center font-semibold hidden md:block">Statistics</div>
  <div class="flex flex-row gap-x-4">
    <GraphNodeStatItem type={GraphNodeStatType.Avg} value={data.avg} />
    <GraphNodeStatItem type={GraphNodeStatType.Min} value={data.min} />
    <GraphNodeStatItem type={GraphNodeStatType.Max} value={data.max} />
    <GraphNodeStatItem type={GraphNodeStatType.Count} value={data.count} />
    <GraphNodeStatItem type={GraphNodeStatType.Sum} value={data.sum} />
  </div>
</div>
