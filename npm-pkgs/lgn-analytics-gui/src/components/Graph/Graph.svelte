<script lang="ts">
  import { onMount } from "svelte";
  import { useLocation } from "svelte-navigator";
  import type { Writable } from "svelte/store";

  import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
  import type { CallTreeNode } from "@lgn/proto-telemetry/dist/calltree";

  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { makeGrpcClient } from "@/lib/client";

  import Loader from "../Misc/Loader.svelte";
  import GraphHeader from "./GraphHeader.svelte";
  import GraphNode from "./GraphNode.svelte";
  import { GraphParameters } from "./Lib/GraphParameters";
  import { GraphState } from "./Store/GraphState";
  import { getGraphStateStore, scopeStore } from "./Store/GraphStateStore";
  import type { NodeStateStore } from "./Store/GraphStateStore";
  import CallGraph from "../CumulatedGraph/CallGraph.svelte";
  import type { CumulativeCallGraphBlockDesc } from "@lgn/proto-telemetry/dist/callgraph";

  const components: Record<number, GraphNode> = {};
  const locationStore = useLocation();

  let client: PerformanceAnalyticsClientImpl;
  let loading = true;
  let beginMsFilter = 0;
  let endMsFilter = 0;
  let processId = "";
  let blocks: CumulativeCallGraphBlockDesc[];
  let startTicks: number;
  let tscFrequency: number;
  let graphState: GraphState;
  let store: Writable<Map<number, NodeStateStore>>;

  onMount(async () => {
    graphState = new GraphState();
    store = graphState.Store;

    loadingStore.reset(1);
    graphState.reset();
    const grpcClient = makeGrpcClient();
    if (grpcClient) {
      client = grpcClient;
      await fetchData();
    }
  });

  async function fetchData() {
    ({
      processId,
      beginMs: beginMsFilter,
      endMs: endMsFilter,
    } = GraphParameters.getGraphParameter($locationStore.search));

    ({ blocks, startTicks, tscFrequency } =
      await client.fetch_cumulative_call_graph_manifest({
        processId: processId,
        beginMs: beginMsFilter,
        endMs: endMsFilter,
      }));

    blocks.forEach(async (blockDesc) => {
      if (!client) {
        return;
      }

      loadingStore.addWork();

      const { callTree, streamHash, streamName } =
        await client.fetch_cumulative_call_graph_block({
          blockId: blockDesc.id,
          tscFrequency,
          startTicks,
          beginMs: beginMsFilter,
          endMs: endMsFilter,
        });

      if (!callTree) {
        return;
      }

      loadingStore.completeWork();

      if (loading) {
        loading = false;
      }

      scopeStore.update((s) => {
        s = { ...s, ...callTree.scopes };
        s[streamHash] = {
          name: `Thread: ${streamName}`,
          hash: streamHash,
          filename: "",
          line: 0,
        };
        return s;
      });

      if (callTree.root) {
        callTree.root.hash = streamHash;
        graphState.Roots.push(callTree.root.hash);

        let range = { begin: Infinity, end: -Infinity };
        computeRange(callTree.root, range);

        let root = graphState.Nodes.get(streamHash);
        if (!root) {
          root = getGraphStateStore(
            streamHash,
            beginMsFilter,
            endMsFilter,
            graphState
          );
          graphState.Nodes.set(streamHash, root);
        }

        root.updateRange(range);
        onNodeReceived(callTree.root, null);
        graphState.tick();
      }
    });
  }

  function computeRange(
    node: CallTreeNode,
    range: { begin: number; end: number }
  ) {
    range.begin = Math.min(range.begin, node.beginMs);
    range.end = Math.max(range.end, node.endMs);
    node.children.forEach((child) => {
      computeRange(child, range);
    });
  }

  function overlaps(node: CallTreeNode) {
    return node.endMs >= beginMsFilter && node.beginMs <= endMsFilter;
  }

  function onNodeReceived(node: CallTreeNode, parent: CallTreeNode | null) {
    if (!overlaps(node)) {
      return;
    }
    let store = graphState.Nodes.get(node.hash);
    if (!store) {
      store = getGraphStateStore(
        node.hash,
        beginMsFilter,
        endMsFilter,
        graphState
      );
      graphState.Nodes.set(node.hash, store);
    }
    store.registerSelfCall(node, parent);
    node.children.forEach((c) => {
      if (store) {
        store.registerChildCall(c);
        onNodeReceived(c, node);
      }
    });
  }

  function onEdgeClicked(e: CustomEvent<{ hash: number }>) {
    components[e.detail.hash]?.setCollapse(false);
  }
</script>

<Loader {loading}>
  <div slot="body" class="flex flex-col">
    <div class="items-end pb-1">
      <GraphHeader {beginMsFilter} {endMsFilter} {blocks} {graphState} />
    </div>
    <div class="flex flex-col overflow-y-auto">
      {#each Array.from($store) as [key, node] (key)}
        <GraphNode
          {node}
          {graphState}
          on:clicked={(e) => onEdgeClicked(e)}
          bind:this={components[key]}
        />
      {/each}
    </div>
    <CallGraph
      begin={beginMsFilter}
      end={endMsFilter}
      {processId}
      debug={false}
    />
  </div>
</Loader>
