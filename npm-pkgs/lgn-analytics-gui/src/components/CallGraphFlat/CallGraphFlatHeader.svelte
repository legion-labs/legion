<script lang="ts">
  import type { CumulatedCallGraphStore } from "@/lib/CallGraph/CallGraphStore";
  import { formatExecutionTime } from "@/lib/format";

  export let store: CumulatedCallGraphStore;
  export let endMsFilter: number;
  export let beginMsFilter: number;
</script>

<div class="text-text text-xs flex flex-row gap-x-2 float-right select-none">
  <div>
    <span class="font-semibold">Selection: </span>
    {formatExecutionTime(endMsFilter - beginMsFilter)}
  </div>
  <div>
    <span class="font-semibold">Cache: </span>{$store.cache.size ?? 0}
  </div>
  <div>
    <span class="font-semibold">Max: </span>
    {formatExecutionTime($store.flatData.getMax())}
  </div>
</div>
