<script lang="ts">
  type GraphParams = {
    processId: string;
    beginMs: number;
    endMs: number;
  };
  
  import { useLocation } from "svelte-navigator";
  import { onMount } from "svelte";
  import {
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
  } from "@/proto/analytics";
  import { Process } from "@/proto/process";

  const locationStore = useLocation();
  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );
  let processInfo: Process | null = null;
  

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

    const { scopes, nodes } = await client.process_cumulative_call_graph({
      process: processInfo,
      beginMs: params.beginMs,
      endMs: params.endMs });
    
    console.log(scopes, nodes);
  }

  onMount(() => {
    fetchData();
  });

  
</script>

<div>
  <h1>Graph</h1>
</div>
