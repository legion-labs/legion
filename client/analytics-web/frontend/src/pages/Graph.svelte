<script lang="ts">
  /* eslint-disable @typescript-eslint/no-non-null-assertion */

  type GraphParams = {
    processId: string;
    beginMs: number;
    endMs: number;
  };

  import { useLocation } from "svelte-navigator";
  import { onMount } from "svelte";
  import {
    CumulativeCallGraphNode,
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
    ScopeDesc,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { Process } from "@lgn/proto-telemetry/codegen/process";
  import { formatExecutionTime } from "@/lib/format";

  const locationStore = useLocation();
  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );
  let processInfo: Process | null = null;
  let scopes: Record<number, ScopeDesc> = {};
  let nodes: CumulativeCallGraphNode[] | null = null;
  let maxSum: number | null = null;
  let selectedNode: CumulativeCallGraphNode | null = null;

  function getUrlParams(): GraphParams {
    const params = new URLSearchParams($locationStore.search);
    const processId = params.get("process");
    if (!processId) {
      throw new Error("missing param process");
    }
    const beginStr = params.get("begin");
    if (!beginStr) {
      throw new Error("missing param begin");
    }
    const endStr = params.get("end");
    if (!endStr) {
      throw new Error("missing param end");
    }
    return {
      processId: processId,
      beginMs: parseFloat(beginStr),
      endMs: parseFloat(endStr),
    };
  }

  async function fetchData() {
    const params = getUrlParams();
    const { process } = await client.find_process({
      processId: params.processId,
    });
    if (!process) {
      throw new Error("Error in client.find_process");
    }
    processInfo = process;

    const reply = await client.process_cumulative_call_graph({
      process: processInfo,
      beginMs: params.beginMs,
      endMs: params.endMs,
    });

    reply.scopes.forEach(function (scope) {
      scopes[scope.hash] = scope;
    });
    nodes = reply.nodes.filter((item) => item.stats && item.hash != 0); //todo: fix this on server side
    nodes = nodes.sort((lhs, rhs) => rhs.stats!.sum - lhs.stats!.sum);
    maxSum = nodes[0].stats!.sum;
  }

  function formatFunDivWidth(node: CumulativeCallGraphNode): string {
    if (!maxSum) {
      return "";
    }
    const pct = (node.stats!.sum * 95) / maxSum;
    return `width:${pct}%`;
  }

  function formatEdgeDivWidth(
    selectedNode: CumulativeCallGraphNode,
    edgeWeight: number
  ): string {
    const pct = (edgeWeight * 95) / selectedNode.stats!.sum;
    return `width:${pct}%`;
  }

  function formatFunLabel(node: CumulativeCallGraphNode): string {
    return scopes[node.hash].name + " " + formatExecutionTime(node.stats!.sum);
  }

  function onFunClick(node: CumulativeCallGraphNode) {
    const funlist = document.getElementById("funlist");
    if (funlist) {
      funlist.style.height = window.innerHeight * 0.4 + "px";
    }
    selectedNode = node;
  }

  function onEdgeClick(hash: number) {
    if (!nodes) {
      return;
    }
    const found = nodes.find((item) => item.hash === hash);
    if (found) {
      selectedNode = found;
    } else {
      selectedNode = null;
    }
  }

  function formatSum(node: CumulativeCallGraphNode): string {
    return formatExecutionTime(node.stats!.sum);
  }

  function formatMin(node: CumulativeCallGraphNode): string {
    return formatExecutionTime(node.stats!.min);
  }

  function formatMax(node: CumulativeCallGraphNode): string {
    return formatExecutionTime(node.stats!.max);
  }

  function formatAvg(node: CumulativeCallGraphNode): string {
    return formatExecutionTime(node.stats!.avg);
  }

  function formatMedian(node: CumulativeCallGraphNode): string {
    return formatExecutionTime(node.stats!.median);
  }

  function formatCount(node: CumulativeCallGraphNode): string {
    return node.stats!.count.toString();
  }

  onMount(() => {
    fetchData();
  });
</script>

<div>
  <h1>Graph</h1>
  {#if nodes}
    <h2>Function List</h2>
    <div id="funlist">
      {#each nodes as node (node.hash)}
        <div
          class="fundiv"
          style={formatFunDivWidth(node)}
          on:click={function (_event) {
            onFunClick(node);
          }}
        >
          <span>
            {formatFunLabel(node)}
          </span>
        </div>
      {/each}
    </div>
  {/if}
  {#if selectedNode}
    <h2>Selected Function</h2>
    <div class="selecteddiv">
      <div>
        <span class="selectedproperty">name </span>
        <span>{scopes[selectedNode.hash].name}</span>
      </div>
      <div>
        <span class="selectedproperty">sum </span>
        <span>{formatSum(selectedNode)}</span>
      </div>
      <div>
        <span class="selectedproperty">min </span>
        <span>{formatMin(selectedNode)}</span>
      </div>
      <div>
        <span class="selectedproperty">max </span>
        <span>{formatMax(selectedNode)}</span>
      </div>
      <div>
        <span class="selectedproperty">average </span>
        <span>{formatAvg(selectedNode)}</span>
      </div>
      <div>
        <span class="selectedproperty">median </span>
        <span>{formatMedian(selectedNode)}</span>
      </div>
      <div>
        <span class="selectedproperty">count </span>
        <span>{formatCount(selectedNode)}</span>
      </div>
    </div>

    <h3>Callees</h3>

    {#each selectedNode.callees as edge (edge.hash)}
      <div
        class="fundiv"
        style={formatEdgeDivWidth(selectedNode, edge.weight)}
        on:click={function (_event) {
          onEdgeClick(edge.hash);
        }}
      >
        <span>
          {scopes[edge.hash].name}
        </span>
      </div>
    {/each}

    <h3>Callers</h3>

    {#each selectedNode.callers as edge (edge.hash)}
      <div
        class="fundiv"
        style={formatEdgeDivWidth(selectedNode, edge.weight)}
        on:click={function (_event) {
          onEdgeClick(edge.hash);
        }}
      >
        <span>
          {scopes[edge.hash].name}
        </span>
      </div>
    {/each}
  {/if}
</div>

<style lang="postcss">
  h1 {
    @apply text-2xl;
  }

  h2 {
    @apply text-xl;
  }

  #funlist {
    overflow-y: auto;
  }

  .selecteddiv {
    margin: 5px;
    text-align: left;
    white-space: nowrap;
  }

  .selectedproperty {
    font-weight: bold;
  }

  .fundiv {
    margin: 5px;
    text-align: left;
    background-color: #fea446;
    overflow: visible;
    white-space: nowrap;
  }

  .fundiv span {
    margin: 0 10px;
  }

  .fundiv:hover {
    color: white;
    background-color: #ca2f0f;
  }

  .fundiv span:hover {
    margin: 0 10px;
    background-color: #ca2f0f;
  }
</style>
