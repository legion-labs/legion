<script lang="ts">
  import {
    GrpcWebImpl,
    MetricDataPoint,
    MetricDesc,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { onMount } from "svelte";
  import { link } from "svelte-navigator";
  import { zoomHorizontalViewRange } from "@/lib/zoom";
  import { formatExecutionTime } from "@/lib/format";
  import {
    DrawSelectedRange,
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
    SelectionState,
  } from "@/lib/time_range_selection";

  type BeginPan = {
    beginMouseX: number;
    viewRange: [number, number];
  };

  export let id: string;

  let beginPan: BeginPan | undefined;
  let canvas: HTMLCanvasElement | undefined;
  let currentSelection: [number, number] | undefined;
  let dataTracks: Record<string, MetricDataPoint[]> = {};
  let maxMs = -Infinity;
  let metrics: MetricDesc[] = [];
  let minMs = Infinity;
  let renderingContext: CanvasRenderingContext2D | undefined;
  let selectionState: SelectionState = NewSelectionState();
  let viewRange: [number, number] | undefined;

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
    DrawSelectedRange(canvas, renderingContext, selectionState, getViewRange());
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

  function onPan(event: MouseEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }

    if (!beginPan) {
      beginPan = {
        beginMouseX: event.offsetX,
        viewRange: getViewRange(),
      };
    }

    const factor =
      (beginPan.viewRange[1] - beginPan.viewRange[0]) / canvas.width;
    const offsetMs = factor * (beginPan.beginMouseX - event.offsetX);

    viewRange = [
      beginPan.viewRange[0] + offsetMs,
      beginPan.viewRange[1] + offsetMs,
    ];
  }

  function onMouseDown(event: MouseEvent) {
    if (RangeSelectionOnMouseDown(event, selectionState)) {
      currentSelection = selectionState.selectedRange;
      drawCanvas();
    }
  }

  // returns if the view should be updated
  function PanOnMouseMove(event: MouseEvent): boolean {
    if (event.buttons !== 1) {
      beginPan = undefined;
      return false;
    }

    if (!event.shiftKey) {
      onPan(event);
      return true;
    }
    return false;
  }

  function onMouseMove(event: MouseEvent) {
    if (!canvas) {
      return;
    }
    if (
      RangeSelectionOnMouseMove(
        event,
        selectionState,
        canvas,
        getViewRange()
      ) ||
      PanOnMouseMove(event)
    ) {
      currentSelection = selectionState.selectedRange;
      drawCanvas();
    }
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
    on:mousemove|preventDefault={onMouseMove}
    on:mousedown|preventDefault={onMouseDown}
  />
  <div id="selected-time-range-div">
    {#if currentSelection}
      <h3>Selected time range</h3>
      <div>
        <span>beginning: </span>
        <span>{formatExecutionTime(currentSelection[0])}<span /></span>
      </div>
      <div>
        <span>end: </span>
        <span>{formatExecutionTime(currentSelection[1])}<span /></span>
      </div>
      <div>
        <span>duration: </span>
        <span
          >{formatExecutionTime(currentSelection[1] - currentSelection[0])}<span
          /></span
        >
      </div>
      <div class="call-graph-link">
        <a
          href={`/cumulative-call-graph?process=${id}&begin=${currentSelection[0]}&end=${currentSelection[1]}`}
          use:link
        >
          Cumulative Call Graph
        </a>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  #metric-selection-div {
    display: inline-block;
  }

  #canvas_plot {
    display: inline-block;
  }

  #selected-time-range-div {
    display: inline-block;
    width: 200px;
    text-align: left;
  }

  .metric-checkbox-div {
    text-align: left;
  }

  #canvas_plot {
    display: inline-block;
    margin: auto;
  }

  .call-graph-link {
    @apply text-[#42b983] underline;
  }
</style>
