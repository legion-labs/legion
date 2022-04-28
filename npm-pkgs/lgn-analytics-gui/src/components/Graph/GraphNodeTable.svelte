<script lang="ts">
  import GraphNodeTableRow from "./GraphNodeTableRow.svelte";
  import { GraphNodeStatType } from "./Lib/GraphNodeStatType";
  import { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";
  import type { NodeStateStore } from "./Store/GraphStateStore";

  export let node: NodeStateStore;
  export let kind: GraphNodeTableKind;

  $: name = kind === GraphNodeTableKind.Callees ? "Callees" : "Callers";
  $: data = kind == GraphNodeTableKind.Callees ? $node.children : $node.parents;
</script>

<table class="text-headline font-thin self-start border-separate">
  {#if data.size > 0}
    <thead class="select-none">
      <tr>
        <td style:width="0" />
        <td>{name}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Avg]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Min]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Max]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Sd]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Count]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Sum]}</td>
      </tr>
    </thead>
    <tbody>
      {#each [...data].sort((a, b) => b[1].acc - a[1].acc) as [key, value] (key)}
        <GraphNodeTableRow {kind} on:clicked {value} {node} />
      {/each}
    </tbody>
  {/if}
</table>

<style lang="postcss">
  table {
    border-spacing: 0px 0.25rem;
  }

  .stat {
    @apply text-center w-20 text-xs;
  }
</style>
