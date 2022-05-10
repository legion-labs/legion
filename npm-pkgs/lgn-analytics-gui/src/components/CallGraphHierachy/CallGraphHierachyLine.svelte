<script lang="ts">
  import type { CallGraphNode } from "@/lib/CallGraph/CallGraphNode";
  import type { CumulatedCallGraphHierarchyStore } from "@/lib/CallGraph/CallGraphStore";
  import { formatExecutionTime } from "@/lib/format";

  export let depth = 0;
  export let node: CallGraphNode;
  export let store: CumulatedCallGraphHierarchyStore;
  export let threadId: number;

  let collapsed = depth > 4;
  let value = node.value;

  function* getChildren() {
    const thread = $store.threads.get(threadId);
    if (!thread) {
      return;
    }
    for (const [k, _] of node.children) {
      const childNode = thread.data.get(k);
      if (childNode) {
        yield childNode;
      }
    }
  }

  const children = Array.from(getChildren()).sort(
    (r, l) => l.value.acc - r.value.acc
  );
</script>

{#if node}
  <div role="row" class="root flex flex-row cursor-pointer text-sm">
    <div
      role="cell"
      style={`padding-left: ${depth + 0.25}rem`}
      class="truncate w-1/2 flex-grow"
      on:click={() => (collapsed = !collapsed)}
    >
      {$store.scopes && $store.scopes[node.hash]?.name}
    </div>
    <div role="cell" class="stat">{value.count.toLocaleString()}</div>
    <div role="cell" class="stat">{formatExecutionTime(value.avg)}</div>
    <div role="cell" class="stat">{formatExecutionTime(value.min)}</div>
    <div role="cell" class="stat">{formatExecutionTime(value.max)}</div>
    <div role="cell" class="stat">{formatExecutionTime(value.sd)}</div>
    <div role="cell" class="stat">{formatExecutionTime(value.acc)}</div>
  </div>
{/if}
{#if !collapsed}
  {#each children as n (n.hash)}
    <svelte:self node={n} {store} {threadId} depth={depth + 1} />
  {/each}
{/if}

<style lang="postcss">
  .root:nth-child(odd) {
    @apply surface;
  }

  .stat {
    @apply text-right text-xs w-28 truncate flex-shrink-0 pr-2;
  }
</style>
