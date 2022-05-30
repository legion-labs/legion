<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { CallGraphNode } from "@/lib/CallGraph/CallGraphNode";
  import type { CallGraphNodeTableKind } from "@/lib/CallGraph/CallGraphNodeTableKind";
  import type { CallGraphNodeValue } from "@/lib/CallGraph/CallGraphNodeValue";
  import type { CumulatedCallGraphFlatStore } from "@/lib/CallGraph/CallGraphStore";
  import { formatExecutionTime } from "@/lib/format";

  export let value: CallGraphNodeValue;
  export let hash: number;
  export let node: CallGraphNode;
  export let kind: CallGraphNodeTableKind;
  export let store: CumulatedCallGraphFlatStore;

  const dispatcher = createEventDispatcher<{
    click: { hash: number };
  }>();

  $: name = $store.scopes[hash]?.name;
  $: fill =
    kind === "callees"
      ? (100 * value.acc) / node.value.acc
      : (100 * value.childSum) / node.value.acc;

  function onClick() {
    dispatcher("click", { hash });
  }
</script>

<tr
  class:bg-graph-red={kind === "callers"}
  class:bg-graph-orange={kind === "callees"}
  class="cursor-pointer text-black relative"
  on:click={onClick}
>
  <div
    style:width={`${fill >= 100 ? 0 : fill}%`}
    class="absolute bg-slate-900 bg-opacity-20 h-full"
  />
  <td class="truncate max-w-md" title={name}>
    {name}
    <span class="text-xs">
      ({formatExecutionTime(value.acc)})
    </span>
  </td>
  <td class="stat">{formatExecutionTime(value.avg)}</td>
  <td class="stat">{formatExecutionTime(value.min)}</td>
  <td class="stat">{formatExecutionTime(value.max)}</td>
  <td class="stat">{formatExecutionTime(value.sd)}</td>
  <td class="stat">{value.count.toLocaleString()}</td>
  <td class="stat">{formatExecutionTime(value.acc)}</td>
</tr>

<style lang="postcss">
  .stat {
    @apply text-center text-xs truncate;
  }
</style>
