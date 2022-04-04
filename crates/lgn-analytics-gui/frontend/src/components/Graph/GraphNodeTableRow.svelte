<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type {
    CallGraphEdge,
    CumulativeCallGraphNode,
  } from "@lgn/proto-telemetry/dist/analytics";

  import { formatExecutionTime } from "@/lib/format";

  import { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";
  import { scopeStore } from "./Store/GraphStore";

  export let edge: CallGraphEdge;
  export let parent: CumulativeCallGraphNode;
  export let kind: GraphNodeTableKind;

  const clickDispatcher = createEventDispatcher<{
    clicked: { hash: number };
  }>();

  $: name = $scopeStore[edge.hash].name;
  $: fill = (edge.weight * 100) / parent.stats!.sum;

  function onClick(_: MouseEvent) {
    clickDispatcher("clicked", { hash: edge.hash });
  }
</script>

<tr
  class:callers={kind === GraphNodeTableKind.Callers}
  class:callees={kind === GraphNodeTableKind.Callees}
  class="cursor-pointer text-black-87 relative"
  on:click={onClick}
>
  <div
    class="absolute bg-slate-900 bg-opacity-20 h-full"
    style:width="{fill}%"
  />
  <td class="truncate">
    {name} ({formatExecutionTime(edge.weight)})
  </td>
  <td class="stat">?</td>
  <td class="stat">?</td>
  <td class="stat">?</td>
  <td class="stat">?</td>
  <td class="stat">?</td>
</tr>

<style lang="postcss">
  .callers {
    @apply bg-graph-red;
  }

  .callees {
    @apply bg-graph-orange;
  }

  .stat {
    @apply text-center;
  }
</style>
