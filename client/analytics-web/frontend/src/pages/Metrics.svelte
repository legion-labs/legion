<script lang="ts">
  import {
    GrpcWebImpl,
    MetricDesc,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { Process } from "@lgn/proto-telemetry/codegen/process";
  import { onMount } from "svelte";

  export let id: string;

  let metrics: MetricDesc[] = [];
  let canvas: HTMLCanvasElement | undefined;
  let renderingContext: CanvasRenderingContext2D | undefined;
  let minMs = Infinity;
  let maxMs = -Infinity;

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  onMount(() => {
    const canvas = document.getElementById("canvas_plot");

    if (!canvas || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error("Canvas can't be found or is invalid");
    }

    const context = canvas.getContext("2d");

    if (!context) {
      throw new Error("Couldn't get context for canvas");
    }

    renderingContext = context;

    fetchProcessInfo();
  });

  async function fetchProcessInfo() {
    const reply = await client.list_process_metrics( {processId: id });
    console.log(reply);
    metrics = reply.metrics;
  }

  function drawCanvas() {
    if (!canvas || !renderingContext) {
      return;
    }

    canvas.height =
      window.innerHeight - canvas.getBoundingClientRect().top - 20;

    renderingContext.clearRect(0, 0, canvas.width, canvas.height);
  }

  function getViewRange(): [number, number] {
    if (viewRange) {
      return viewRange;
    }

    return [minMs, maxMs];
  }

  function onMetricSelectionChanged( name:string, selected:bool ){
    console.log(name, selected);
  }

</script>

<div>
  <div id="metric-selection-div">
    {#each metrics as metric (metric.name)}
      <input type="checkbox" id={metric.name+'_select'}
             on:click={evt => onMetricSelectionChanged(metric.name, evt.srcElement.checked)}
             />
      <label for={metric.name+'_select'}> {metric.name}</label>
    {/each}
  </div>
  <canvas
    bind:this={canvas}
    id="canvas_plot"
    width="1024px"
    />
</div>

<style lang="postcss">

  #metric-selection-div {
    display: inline-block;
  }
  
  #canvas_plot {
    display: inline-block;
    margin: auto;
  }
</style>
