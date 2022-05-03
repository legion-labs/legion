<script lang="ts">
  import { CallGraphNodeStatType } from "@/lib/CallGraph/CallGraphNodeStatType";
  import type { CallGraphNodeValue } from "@/lib/CallGraph/CallGraphNodeValue";

  import CallGraphFlatNodeStatItem from "./CallGraphFlatNodeStatItem.svelte";

  export let node: CallGraphNodeValue;
</script>

<div class="flex flex-row justify-between select-none h-8 headline">
  <div class="items-center font-semibold hidden md:block">Statistics</div>
  {#if node}
    <div class="flex flex-row gap-x-4">
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Avg}
        value={node.avg}
      />
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Min}
        value={node.min}
      />
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Max}
        value={node.max}
      />
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Count}
        value={node.count}
      />
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Sd}
        value={node.sd}
      />
      <CallGraphFlatNodeStatItem
        type={CallGraphNodeStatType.Sum}
        value={node.acc}
      />
    </div>
  {/if}
</div>
