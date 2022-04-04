<script lang="ts">
  import { tick } from "svelte";

  import type { CumulativeCallGraphNode } from "@lgn/proto-telemetry/dist/analytics";

  import { formatExecutionTime } from "@/lib/format";

  import GraphNodeStat from "./GraphNodeStat.svelte";
  import GraphNodeTable from "./GraphNodeTable.svelte";
  import { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";
  import { graphStateStore, scopeStore } from "./Store/GraphStore";

  export let node: CumulativeCallGraphNode;
  export let collapsed = false;

  let div: HTMLElement;

  $: desc = $scopeStore[node.hash];
  // @ts-ignore
  $: fill = (node.stats?.sum ?? 0) / $graphStateStore.MaxSum;

  export async function setCollapse(value: boolean) {
    collapsed = value;
    await tick();
    if (!collapsed) {
      div.scrollIntoView({ behavior: "auto", block: "start" });
    }
  }
</script>

<div class="text-sm py-1 rounded-lg overflow-x-hidden" bind:this={div}>
  <div
    class="flex justify-between select-none cursor-pointer relative bg-skin-700 text-content-87"
    on:click={(_) => setCollapse(!collapsed)}
  >
    {#if desc}
      <div
        class="text-left pl-2 py-1 whitespace-nowrap bg-skin-700"
        style:width="{fill}%"
      >
        <i
          class={`bi bi-chevron-${
            collapsed ? "down" : "up"
          } text-xs text-content-100`}
        />
        {desc.name}
        ({formatExecutionTime(node.stats?.sum ?? 0)})
      </div>
      <div
        class="text-xs text-content-38 absolute pt-1.5 pr-2 right-0"
        class:hidden={!collapsed}
      >
        {(node.stats?.count ?? 0).toLocaleString()} call{(node.stats?.count ??
          0) >= 2
          ? "s"
          : ""}
      </div>
    {/if}
  </div>
  {#if !collapsed && node.stats}
    <div class="bg-skin-700 flex flex-col p-3">
      <GraphNodeStat data={node.stats} />
      <div class="hidden md:block pb-4">
        <div class="w-full border-t border-charcoal-600" />
      </div>
      <div class="hidden md:grid tables gap-2">
        <GraphNodeTable
          on:clicked
          name="Callers"
          data={node.callers}
          parent={node}
          kind={GraphNodeTableKind.Callers}
        />
        <GraphNodeTable
          on:clicked
          name="Callees"
          data={node.callees}
          parent={node}
          kind={GraphNodeTableKind.Callees}
        />
      </div>
    </div>
  {/if}
</div>

<style lang="postcss">
  .small {
  }

  .tables {
    grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
  }
</style>
