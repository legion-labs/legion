<script lang="ts">
  import { tick } from "svelte";

  import type { CallGraphNode } from "@/lib/CallGraph/CallGraphNode";
  import { CallGraphNodeTableKind } from "@/lib/CallGraph/CallGraphNodeTableKind";
  import type { CumulatedCallGraphFlatStore } from "@/lib/CallGraph/CallGraphStore";
  import { formatExecutionTime } from "@/lib/format";

  import CallGraphFlatNodeStat from "./CallGraphFlatNodeStat.svelte";
  import CallGraphFlatNodeTable from "./CallGraphFlatNodeTable.svelte";

  export let store: CumulatedCallGraphFlatStore;
  export let node: CallGraphNode;
  export let collapsed = true;

  // @ts-ignore
  $: fill = (100 * node.value.acc) / $store.getMax();
  $: desc = $store?.scopes[node.hash];

  export async function setCollapse(value: boolean) {
    collapsed = value;
    await tick();
  }
</script>

<div class="text-sm py-1 rounded-lg overflow-x-hidden">
  <div
    class="flex justify-between select-none cursor-pointer relative surface headline"
    on:click={(_) => setCollapse(!collapsed)}
  >
    {#if desc}
      <div
        class="text-left pl-2 py-1 whitespace-nowrap background"
        style:width="{fill}%"
      >
        <i class={`bi bi-chevron-${collapsed ? "down" : "up"} text-xs`} />
        {desc.name}
        <span class="text-xs placeholder">
          ({formatExecutionTime(node.value.acc)})
        </span>
      </div>
      <div
        class="text-xs placeholder absolute pt-1.5 pr-2 right-0"
        class:hidden={!collapsed}
      >
        {node.value.count.toLocaleString()} call{node.value.count >= 2
          ? "s"
          : ""}
      </div>
    {/if}
  </div>
  {#if !collapsed && node}
    <div class="background flex flex-col p-3">
      <CallGraphFlatNodeStat node={node.value} />
      <div class="hidden md:block pb-4">
        <div class="w-full border-t border-headline" />
      </div>
      <div class="hidden md:grid tables gap-2">
        <CallGraphFlatNodeTable
          on:clicked
          {node}
          kind={CallGraphNodeTableKind.Callers}
          {store}
        />
        <CallGraphFlatNodeTable
          on:clicked
          {node}
          kind={CallGraphNodeTableKind.Callees}
          {store}
        />
      </div>
    </div>
  {/if}
</div>

<style lang="postcss">
  .tables {
    grid-template-columns: repeat(auto-fit, minmax(1000px, 1fr));
  }
</style>
