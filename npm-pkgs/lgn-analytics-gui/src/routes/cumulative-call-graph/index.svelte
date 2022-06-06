<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { getContext } from "svelte";
  import { tick } from "svelte";

  import GraphHeader from "@/components/CallGraphFlat/CallGraphFlatHeader.svelte";
  import GraphNode from "@/components/CallGraphFlat/CallGraphFlatNode.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import { CallGraphParameters } from "@/lib/CallGraph/CallGraphParameters";
  import { getProcessCumulatedCallGraphFlat } from "@/lib/CallGraph/CallGraphStore";
  import type { CumulatedCallGraphFlatStore } from "@/lib/CallGraph/CallGraphStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";

  const components: Record<number, GraphNode> = {};
  const client = getContext("http-client");

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

  async function onEdgeClicked({
    detail: { hash },
  }: CustomEvent<{ hash: number }>) {
    const component = components[hash];

    if (!component) {
      return;
    }

    component.setCollapse(false);
    await tick();
    component.scrollTo();
  }
</script>

<Layout>
  <div slot="content">
    {#if loading}
      <Loader />
    {:else}
      <div class="cumulative-call-graph">
        <div class="items-end pb-1">
          <GraphHeader {beginMsFilter} {endMsFilter} {store} />
        </div>
        <div class="flex flex-col">
          {#each Array.from($store.nodes.values()).sort((a, b) => b.value.acc - a.value.acc) as node (node.hash)}
            <GraphNode
              {node}
              {store}
              on:click={onEdgeClicked}
              bind:this={components[node.hash]}
            />
          {/each}
        </div>
      </div>
    {/if}
  </div>
</Layout>

<style lang="postcss">
  .cumulative-call-graph {
    @apply flex flex-col pt-4 pb-1 px-2;
  }
</style>
