<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import { formatExecutionTime } from "@/lib/format";

  import { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";
  import type { GraphStateNode } from "./Store/GraphStateNode";
  import type { NodeStateStore } from "./Store/GraphStateStore";
  import { scopeStore } from "./Store/GraphStateStore";

  export let value: GraphStateNode;
  export let node: NodeStateStore;
  export let kind: GraphNodeTableKind;

  const clickDispatcher = createEventDispatcher<{
    clicked: { hash: number };
  }>();

  $: name = $scopeStore[value.hash]?.name;
  // @ts-ignore
  $: fill =
    kind === GraphNodeTableKind.Callees
      ? (100 * value.acc) / $node.acc
      : (100 * value.childWeight) / $node.acc;

  function onClick(_: MouseEvent) {
    clickDispatcher("clicked", { hash: value.hash });
  }
</script>

<tr
  class:callers={kind === GraphNodeTableKind.Callers}
  class:callees={kind === GraphNodeTableKind.Callees}
  class="cursor-pointer text-black-87 relative"
  on:click={onClick}
>
  <div
    style:width="{fill >= 100 ? 0 : fill}%"
    class="absolute bg-slate-900 bg-opacity-20 h-full"
  />
  <td class="truncate">
    {name}
    ({formatExecutionTime(value.acc)})
  </td>
  <td class="stat">{formatExecutionTime(value.avg)}</td>
  <td class="stat">{formatExecutionTime(value.min)}</td>
  <td class="stat">{formatExecutionTime(value.max)}</td>
  <td class="stat">{formatExecutionTime(value.sd)}</td>
  <td class="stat">{value.count.toLocaleString()}</td>
  <td class="stat">{formatExecutionTime(value.acc)}</td>
</tr>

<style lang="postcss">
  .callers {
    @apply bg-graph-red;
  }

  .callees {
    @apply bg-graph-orange;
  }

  .stat {
    @apply text-center text-xs truncate;
  }
</style>
