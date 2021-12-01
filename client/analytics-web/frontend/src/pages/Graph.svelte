<script lang="ts">
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

  
  function getUrlParams() :GraphParams {
    const params = new URLSearchParams($locationStore.search);
    const processId = params.get("process")
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
      processId : processId,
      beginMs: parseFloat(beginStr),
      endMs: parseFloat(endStr)
    };
  }

  async function fetchData() {
    const params = getUrlParams();
    const { process } = await client.find_process({ processId: params.processId });
    if (!process) {
      throw new Error("Error in client.find_process");
    }
    processInfo = process;

    const reply = await client.process_cumulative_call_graph({
      process: processInfo,
      beginMs: params.beginMs,
      endMs: params.endMs });

    reply.scopes.forEach( function(scope){
      scopes[scope.hash] = scope;
    } );
    nodes = reply.nodes.filter( item => item.stats && item.hash != 0 ); //todo: fix this on server side
    nodes = nodes.sort( (lhs, rhs) => rhs.stats!.sum - lhs.stats!.sum );
    maxSum = nodes[0].stats!.sum;
  }

  function formatFunDivWidth(node: CumulativeCallGraphNode) : string{
    if (!maxSum) {
      return "";
    }
    const pct = node.stats!.sum * 100 / maxSum;
    return `width:${pct*0.95}%`;
  }

  function formatFunLabel(node: CumulativeCallGraphNode) : string{
    return scopes[node.hash].name + ' ' + formatExecutionTime(node.stats!.sum);
  }

  onMount(() => {
    fetchData();
  });

  
</script>

<div>
  <h1>Graph</h1>
  {#if nodes}
    <h2>Function List</h2>
    {#each nodes as node (node.hash)}
      <div class="fundiv" style={formatFunDivWidth(node)}>
        <span>
          {formatFunLabel(node)}
        </span>
      </div>
    {/each}
  {/if}
</div>

<style lang="postcss">

  .fundiv {
    margin: 5px;
    text-align: left;
    background-color: rgba(64, 64, 200, 0.1);
    overflow: visible;
    white-space: nowrap;
  }

  .fundiv span {
    margin: 0 10px;
  }
  
  .fundiv:hover {
    color: white;
    background-color: rgba(64, 64, 200, 1.0);
  }

  .fundiv span:hover {
    margin: 0 10px;
    background-color: rgba(64, 64, 200, 1.0);
  }
  
  
</style>
