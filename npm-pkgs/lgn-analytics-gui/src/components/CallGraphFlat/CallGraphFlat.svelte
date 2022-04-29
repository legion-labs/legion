<script lang="ts">
  import { onMount } from "svelte";
  import { useLocation } from "svelte-navigator";

  import { CallGraphParameters } from "@/lib/CallGraph/CallGraphParameters";
  import { getProcessCumulatedCallGraph } from "@/lib/CallGraph/CallGraphStore";
  import type { CumulatedCallGraphStore } from "@/lib/CallGraph/CallGraphStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";

  import Loader from "../Misc/Loader.svelte";
  import GraphHeader from "./CallGraphFlatHeader.svelte";
  import GraphNode from "./CallGraphFlatNode.svelte";

  const components: Record<number, GraphNode> = {};
  const locationStore = useLocation();

  let beginMsFilter = 0;
  let endMsFilter = 0;
  let processId = "";
  let store: CumulatedCallGraphStore;

  $: loading = store ? $store.loading : true;

  onMount(async () => {
    ({
      processId,
      beginMs: beginMsFilter,
      endMs: endMsFilter,
    } = CallGraphParameters.getGraphParameter($locationStore.search));

    loadingStore.reset(1);

    store = await getProcessCumulatedCallGraph(
      processId,
      beginMsFilter,
      endMsFilter,
      loadingStore
    );
  });

  function onEdgeClicked(e: CustomEvent<{ hash: number }>) {
    components[e.detail.hash]?.setCollapse(false);
  }
</script>

<Loader {loading}>
  <div slot="body" class="flex flex-col">
    <div class="items-end pb-1">
      <GraphHeader {beginMsFilter} {endMsFilter} {store} />
    </div>
    <div class="flex flex-col overflow-y-auto">
      {#each Array.from($store.flatData.nodes.values()).sort((a, b) => b.value.acc - a.value.acc) as node (node.hash)}
        <GraphNode
          {node}
          {store}
          on:clicked={(e) => onEdgeClicked(e)}
          bind:this={components[node.hash]}
        />
      {/each}
    </div>
  </div>
</Loader>
