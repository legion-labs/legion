<script lang="ts">
  import type { CumulatedCallGraphStore } from "@/components/CumulatedGraph/Lib/CallGraphStore";
  import { formatExecutionTime } from "@/lib/format";

  import type { CallGraphNode } from "./Lib/CallGraphNode";

  export let depth = 0;
  export let node: CallGraphNode;
  export let store: CumulatedCallGraphStore;
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
</script>

{#if node}
  <tr class="cursor-pointer text-sm">
    <td
      style={`padding-left: ${depth * 20}px;`}
      on:click={(_) => (collapsed = !collapsed)}
      >{$store.scopes && $store.scopes[node.hash]?.name}
    </td>
    <td class="stat">{value.count}</td>
    <td class="stat">{formatExecutionTime(value.avg)}</td>
    <td class="stat">{formatExecutionTime(value.min)}</td>
    <td class="stat">{formatExecutionTime(value.max)}</td>
    <td class="stat">{formatExecutionTime(value.sd)}</td>
    <td class="stat">{formatExecutionTime(value.acc)}</td>
  </tr>
{/if}
{#if !collapsed}
  {#each Array.from(getChildren()).sort((r, l) => l.value.acc - r.value.acc) as n (n.hash)}
    <svelte:self node={n} {store} {threadId} depth={depth + 1} />
  {/each}
{/if}

<style lang="postcss">
  .stat {
    @apply text-center text-xs;
  }

  tr:nth-child(even) {
    @apply bg-skin-600;
  }
</style>
