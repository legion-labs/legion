<script lang="ts">
  import type { CumulativeCallGraphBlockDesc } from "@lgn/proto-telemetry/dist/callgraph";

  import { formatExecutionTime } from "@/lib/format";

  import type { GraphState } from "./Store/GraphState";

  export let endMsFilter: number;
  export let beginMsFilter: number;
  export let blocks: CumulativeCallGraphBlockDesc[];
  export let graphState: GraphState;
  $: max = graphState.Max;
</script>

<div class="text-text text-xs flex flex-row gap-x-2 float-right select-none">
  <div>
    <span class="font-semibold">Selection: </span>
    {formatExecutionTime(endMsFilter - beginMsFilter)}
  </div>
  <div>
    <span class="font-semibold">Blocks: </span>{blocks?.length ?? 0}
  </div>
  <div>
    <span class="font-semibold">Max: </span>
    {formatExecutionTime($max)}
  </div>
</div>
