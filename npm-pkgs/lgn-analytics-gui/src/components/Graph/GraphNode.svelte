<script lang="ts">
  import { tick } from "svelte";

  import { formatExecutionTime } from "@/lib/format";

  import GraphNodeStat from "./GraphNodeStat.svelte";
  import GraphNodeTable from "./GraphNodeTable.svelte";
  import { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";
  import type { GraphState } from "./Store/GraphState";
  import type { NodeStateStore } from "./Store/GraphStateStore";
  import { scopeStore } from "./Store/GraphStateStore";

  export let node: NodeStateStore;
  export let collapsed = true;
  export let graphState: GraphState;

  $: max = graphState.Max;
  $: desc = $scopeStore[$node?.hash];
  // @ts-ignore
  $: fill = (100 * $node.acc) / $max;

  export async function setCollapse(value: boolean) {
    collapsed = value;
    await tick();
  }
</script>

<div class="text-sm py-1 rounded-lg overflow-x-hidden">
  <div
    class="flex justify-between select-none cursor-pointer relative bg-surface text-content-87"
    on:click={(_) => setCollapse(!collapsed)}
  >
    {#if desc}
      <div
        class="text-left pl-2 py-1 whitespace-nowrap bg-background"
        style:width="{fill}%"
      >
        <i
          class={`bi bi-chevron-${
            collapsed ? "down" : "up"
          } text-xs text-content-100`}
        />
        {desc.name}
        <span class="text-xs text-content-38">
          ({formatExecutionTime($node.acc)})
        </span>
      </div>
      <div
        class="text-xs text-content-38 absolute pt-1.5 pr-2 right-0"
        class:hidden={!collapsed}
      >
        {$node.count.toLocaleString()} call{$node.count >= 2 ? "s" : ""}
      </div>
    {/if}
  </div>
  {#if !collapsed && $node}
    <div class="bg-background flex flex-col p-3">
      <GraphNodeStat {node} />
      <div class="hidden md:block pb-4">
        <div class="w-full border-t border-charcoal-600" />
      </div>
      <div class="hidden md:grid tables gap-2">
        <GraphNodeTable on:clicked {node} kind={GraphNodeTableKind.Callers} />
        <GraphNodeTable on:clicked {node} kind={GraphNodeTableKind.Callees} />
      </div>
    </div>
  {/if}
</div>

<style lang="postcss">
  .tables {
    grid-template-columns: repeat(auto-fit, minmax(1000px, 1fr));
  }
</style>
