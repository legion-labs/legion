<script lang="ts">
  import type { CallGraphNode } from "@/lib/CallGraph/CallGraphNode";
  import { CallGraphNodeStatType } from "@/lib/CallGraph/CallGraphNodeStatType";
  import { CallGraphNodeTableKind } from "@/lib/CallGraph/CallGraphNodeTableKind";
  import type { CumulatedCallGraphStore } from "@/lib/CallGraph/CallGraphStore";

  import GraphNodeTableRow from "./CallGraphFlatNodeTableRow.svelte";

  export let node: CallGraphNode;
  export let kind: CallGraphNodeTableKind;
  export let store: CumulatedCallGraphStore;

  $: data =
    kind == CallGraphNodeTableKind.Callees ? node.children : node.parents;
</script>

<table class="text-headline font-thin self-start border-separate">
  {#if data.size > 0}
    <thead class="select-none">
      <tr>
        <td style:width="0" />
        <td
          >{kind === CallGraphNodeTableKind.Callees ? "Callees" : "Callers"}</td
        >
        <td class="stat">{CallGraphNodeStatType[CallGraphNodeStatType.Avg]}</td>
        <td class="stat">{CallGraphNodeStatType[CallGraphNodeStatType.Min]}</td>
        <td class="stat">{CallGraphNodeStatType[CallGraphNodeStatType.Max]}</td>
        <td class="stat">{CallGraphNodeStatType[CallGraphNodeStatType.Sd]}</td>
        <td class="stat"
          >{CallGraphNodeStatType[CallGraphNodeStatType.Count]}</td
        >
        <td class="stat">{CallGraphNodeStatType[CallGraphNodeStatType.Sum]}</td>
      </tr>
    </thead>
    <tbody>
      {#each [...data] as [key, value] (key)}
        <GraphNodeTableRow
          hash={key}
          {kind}
          on:clicked
          {value}
          {node}
          {store}
        />
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
