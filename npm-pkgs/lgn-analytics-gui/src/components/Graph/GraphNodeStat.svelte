<script lang="ts">
  import GraphNodeStatItem from "./GraphNodeStatItem.svelte";
  import { GraphNodeStatType } from "./Lib/GraphNodeStatType";
  import type { NodeStateStore } from "./Store/GraphStateStore";

  export let node: NodeStateStore;
</script>

<div class="flex flex-row justify-between select-none h-8 text-headline">
  <div class="items-center font-semibold hidden md:block">Statistics</div>
  {#if node}
    <div class="flex flex-row gap-x-4">
      <GraphNodeStatItem type={GraphNodeStatType.Avg} value={$node.avg} />
      <GraphNodeStatItem type={GraphNodeStatType.Min} value={$node.min} />
      <GraphNodeStatItem type={GraphNodeStatType.Max} value={$node.max} />
      <GraphNodeStatItem type={GraphNodeStatType.Count} value={$node.count} />
      <GraphNodeStatItem type={GraphNodeStatType.Sd} value={$node.sd} />
      <GraphNodeStatItem type={GraphNodeStatType.Sum} value={$node.acc} />
    </div>
  {/if}
</div>
