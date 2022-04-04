<script lang="ts">
  import type {
    CallGraphEdge,
    CumulativeCallGraphNode,
  } from "@lgn/proto-telemetry/dist/analytics";

  import GraphNodeTableRow from "./GraphNodeTableRow.svelte";
  import { GraphNodeStatType } from "./Lib/GraphNodeStatType";
  import type { GraphNodeTableKind } from "./Lib/GraphNodeTableKind";

  export let name: string;
  export let data: CallGraphEdge[];
  export let parent: CumulativeCallGraphNode;
  export let kind: GraphNodeTableKind;
</script>

<table class="text-content-87 font-thin self-start border-separate">
  {#if data.length > 0}
    <thead class="select-none">
      <tr>
        <td style:width="0" />
        <td>{name}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Avg]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Min]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Max]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Count]}</td>
        <td class="stat">{GraphNodeStatType[GraphNodeStatType.Sum]}</td>
      </tr>
    </thead>
    <tbody>
      {#each data as edge (edge.hash)}
        <GraphNodeTableRow {kind} on:clicked {edge} {parent} />
      {/each}
    </tbody>
  {/if}
</table>

<style lang="postcss">
  table {
    border-spacing: 0px 0.25rem;
  }

  .stat {
    @apply text-center;
    @apply w-12;
  }
</style>
