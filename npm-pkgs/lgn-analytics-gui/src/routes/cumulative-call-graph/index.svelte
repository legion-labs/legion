<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  import GraphHeader from "@/components/CallGraphFlat/CallGraphFlatHeader.svelte";
  import GraphNode from "@/components/CallGraphFlat/CallGraphFlatNode.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import { getHttpClientContext } from "@/contexts";
  import { CallGraphParameters } from "@/lib/CallGraph/CallGraphParameters";
  import { getProcessCumulatedCallGraphFlat } from "@/lib/CallGraph/CallGraphStore";
  import type { CumulatedCallGraphFlatStore } from "@/lib/CallGraph/CallGraphStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";

  const components: Record<number, GraphNode> = {};
  const client = getHttpClientContext();

  let beginMsFilter = 0;
  let endMsFilter = 0;
  let processId = "";
  let store: CumulatedCallGraphFlatStore;

  $: loading = store ? $store.loading : true;

  onMount(async () => {
    ({
      processId,
      beginMs: beginMsFilter,
      endMs: endMsFilter,
    } = CallGraphParameters.getGraphParameter($page.url.search));

    loadingStore.reset(1);

    store = await getProcessCumulatedCallGraphFlat(
      client,
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
      {#each Array.from($store.nodes.values()).sort((a, b) => b.value.acc - a.value.acc) as node (node.hash)}
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
