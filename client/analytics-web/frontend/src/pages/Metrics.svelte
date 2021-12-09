<script lang="ts">
  import {
    GrpcWebImpl,
    MetricDataPoint,
    MetricDesc,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { onMount } from "svelte";
  import { zoomHorizontalViewRange } from "@/lib/zoom";

  export let id: string;

  let metrics: MetricDesc[] = [];
  let canvas: HTMLCanvasElement | undefined;
  let renderingContext: CanvasRenderingContext2D | undefined;
  let minMs = Infinity;
  let maxMs = -Infinity;
  let viewRange: [number, number] | undefined;
  let dataTracks: Record<string, MetricDataPoint[]> = {};

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  function getViewRange(): [number, number] {
    if (viewRange) {
      return viewRange;
    }

    return [minMs, maxMs];
  }

  onMount(() => {
    canvas = document.getElementById("canvas_plot") as HTMLCanvasElement;
    if (!canvas) {
      throw new Error("Canvas can't be found or is invalid");
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Couldn't get context for canvas");
    }
    renderingContext = ctx;

    fetchProcessInfo();
    drawCanvas();
  });

  async function fetchProcessInfo() {
    const reply = await client.list_process_metrics({ processId: id });
    metrics = reply.metrics;
    minMs = reply.minTimeMs;
    maxMs = reply.maxTimeMs;
  }

  function drawTrack(points: MetricDataPoint[]) {
    if (points.length == 0) {
      return;
    }
    if (!canvas || !renderingContext) {
      return;
    }
    let minValue = Infinity;
    let maxValue = -Infinity;
    const viewRange = getViewRange();
    const beginView = viewRange[0];
    const endView = viewRange[1];

    let beginIndex = points.length;
    let endIndex = 0;
    for (let i = 0; i < points.length; ++i) {
      const pt = points[i];
      if (pt.timeMs < beginView) {
        continue;
      } else if (pt.timeMs <= endView) {
        minValue = Math.min(minValue, pt.value);
        maxValue = Math.max(maxValue, pt.value);
        beginIndex = Math.min(beginIndex, i);
        endIndex = Math.max(endIndex, i);
      } else {
        break;
      }
    }

    beginIndex = Math.max(0, beginIndex - 1);
    endIndex = Math.min(endIndex + 2, points.length);

    const timeSpan = endView - beginView;
    const valueSpan = maxValue - minValue;
    const widthPixels = canvas.width;
    const heightPixels = canvas.height;

    renderingContext.beginPath();
    {
      let p = points[beginIndex];
      let x = ((p.timeMs - beginView) / timeSpan) * widthPixels;
      let y = heightPixels - ((p.value - minValue) * heightPixels) / valueSpan;
      renderingContext.moveTo(x, y);
    }
    for (let i = beginIndex + 1; i < endIndex; ++i) {
      let p = points[i];
      let x = ((p.timeMs - beginView) / timeSpan) * widthPixels;
      let y = heightPixels - ((p.value - minValue) * heightPixels) / valueSpan;
      renderingContext.lineTo(x, y);
    }
    renderingContext.lineWidth = 2;
    renderingContext.stroke();
  }

  function drawCanvas() {
    if (!canvas || !renderingContext) {
      return;
    }

    canvas.height =
      window.innerHeight - canvas.getBoundingClientRect().top - 20;

    renderingContext.clearRect(0, 0, canvas.width, canvas.height);
    for (let key in dataTracks) {
      drawTrack(dataTracks[key]);
    }
  }

  async function onMetricSelectionChanged(
    metricName: string,
    evt: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    const selected = evt.currentTarget.checked;
    if (!selected) {
      delete dataTracks[metricName];
      drawCanvas();
      return;
    }
    const reply = await client.fetch_process_metric({
      processId: id,
      metricName: metricName,
      beginMs: minMs,
      endMs: maxMs,
    });
    dataTracks[metricName] = reply.points;
    drawCanvas();
  }

  function onZoom(event: WheelEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }
    viewRange = zoomHorizontalViewRange(getViewRange(), canvas.width, event);
    drawCanvas();
  }
</script>

<div>
  <div id="metric-selection-div">
    {#each metrics as metric (metric.name)}
      <div class="metric-checkbox-div">
        <input
          type="checkbox"
          id={metric.name + "_select"}
          on:click={(evt) => onMetricSelectionChanged(metric.name, evt)}
        />
        <label for={metric.name + "_select"}> {metric.name}</label>
      </div>
    {/each}
  </div>
  <canvas
    bind:this={canvas}
    id="canvas_plot"
    width="1024px"
    on:wheel|preventDefault={onZoom}
  />
</div>

<style lang="postcss">
  #metric-selection-div {
    display: inline-block;
  }

  .metric-checkbox-div {
    text-align: left;
  }

  #canvas_plot {
    display: inline-block;
    margin: auto;
  }
</style>
