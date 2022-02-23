<script lang="ts">
  import { makeGrpcClient } from "@/lib/client";
  import { formatExecutionTime } from "@/lib/format";
  import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
  import * as d3 from "d3";
  import { onDestroy, onMount } from "svelte";
  import { get, Unsubscriber, Writable } from "svelte/store";
  import TimeRangeDetails from "./TimeRangeDetails.svelte";
  import log from "@lgn/web-client/src/lib/log";
  import { Point } from "@/lib/Metric/MetricPoint";
  import { MetricStreamer } from "@/lib/Metric/MetricStreamer";
  import { MetricState } from "@/lib/Metric/MetricState";
  import { getLodFromPixelSizeNs } from "@/lib/lod";
  import MetricSelection from "./Metric/MetricSelection.svelte";
  export let id: string;

  let metricStreamer: MetricStreamer;
  let metricStore: Writable<MetricState[]>;
  const margin = { top: 20, right: 50, bottom: 60, left: 70 };

  const outerHeight = 600;
  const height = outerHeight - margin.top - margin.bottom;

  let mainWidth = 0;
  $: width = mainWidth - margin.left - margin.right;

  let client: PerformanceAnalyticsClientImpl | null = null;
  let totalMinTick = NaN;
  let totalMaxTick = NaN;
  let currentMinTick = NaN;
  let currentMaxTick = NaN;
  let brushStart = -Infinity;
  let brushEnd = Infinity;
  let points: {
    points: Point[];
    name: string;
  }[] = [];
  let loading = true;
  let updateTime: number;
  let lod: number;
  let deltaMs: number;
  let pixelSizeNs: number;

  let x: d3.ScaleLinear<number, number, never>;
  let y: d3.ScaleLinear<number, number, never>;

  /* eslint-disable @typescript-eslint/no-explicit-any */
  let gxAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let gyAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let container: d3.Selection<d3.BaseType, unknown, HTMLElement, any>;
  let brush: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  /* eslint-enable @typescript-eslint/no-explicit-any */

  let xAxis: d3.Axis<d3.NumberValue>;
  let yAxis: d3.Axis<d3.NumberValue>;
  let zoom: d3.ZoomBehavior<Element, unknown>;

  let context: CanvasRenderingContext2D;
  let transform: d3.ZoomTransform = d3.zoomIdentity;
  let canvas: HTMLCanvasElement;
  let pointSubscription: Unsubscriber | undefined;

  $: {
    if (mainWidth && transform) {
      updateChart();
    }
  }

  const getDeltaMs = () =>
    metricStreamer.getTickOffsetMs(currentMaxTick) -
    metricStreamer.getTickOffsetMs(currentMinTick);
  const getPixelSizeNs = () => (getDeltaMs() * 1_000_000) / width;

  onMount(async () => {
    client = await makeGrpcClient();
    await fetchMetricsAsync().then(() => (loading = false));
    createChart();
    updateChart();
  });

  onDestroy(() => {
    const element = document.getElementById("metric-canvas");
    if (element) {
      element.replaceChildren();
    }
    if (pointSubscription) {
      pointSubscription();
    }
  });

  function hashString(string: string): number {
    let hash = 0;
    for (let i = 0; i < string.length; i++) {
      hash = string.charCodeAt(i) + ((hash << 5) - hash);
      hash = hash & hash;
    }
    return hash;
  }

  function updateLod() {
    if (x) {
      const scaleX = transform.rescaleX(x);
      currentMinTick = scaleX.domain()[0].valueOf();
      currentMaxTick = scaleX.domain()[1].valueOf();
    }
    deltaMs = getDeltaMs();
    pixelSizeNs = getPixelSizeNs();
    lod = getLodFromPixelSizeNs(pixelSizeNs);
    metricStreamer?.tick(lod, currentMinTick, currentMaxTick);
    updatePoints(get(metricStore));
  }

  async function fetchMetricsAsync() {
    if (!client) {
      log.error("no client in fetchMetricsAsync");
      return;
    }

    metricStreamer = new MetricStreamer(id);
    metricStore = metricStreamer.metricStore;
    await metricStreamer.initializeAsync();

    totalMinTick = currentMinTick = metricStreamer.minTick;
    totalMaxTick = currentMaxTick = metricStreamer.maxTick;

    pointSubscription = metricStore.subscribe((metricStates) => {
      updatePoints(metricStates);
      updateChart();
    });
  }

  function updatePoints(states: MetricState[]) {
    points = states
      .filter((m) => m.enabled)
      .map((m) => {
        return {
          points: Array.from(
            m.getViewportPoints(currentMinTick, currentMaxTick, lod)
          ),
          name: m.name,
        };
      });
  }

  function refreshZoom() {
    const extent = [width, outerHeight] as [number, number];
    zoom.translateExtent([[0, 0], extent]);
    zoom.extent([[0, 0], extent]);
  }

  function createChart() {
    container = d3.select("#metric-canvas");

    const svgGroup = container
      .append("svg")
      .append("g")
      .attr("transform", `translate(${margin.left}, ${margin.top})`);

    const canvasChart = container
      .append("canvas")
      .style("position", "absolute")
      .style("top", 0)
      .style("left", 0)
      .style("margin-left", `${margin.left}px`)
      .style("margin-top", `${margin.top}px`)
      .style("pointer-events", "none");

    canvas = canvasChart.node() as HTMLCanvasElement;

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    context = canvas.getContext("2d")!;

    x = d3.scaleLinear().domain([totalMinTick, totalMaxTick]);
    y = d3.scaleLinear().nice();

    xAxis = d3
      .axisBottom(x)
      .tickFormat((t) =>
        formatExecutionTime(metricStreamer.getTickRawMs(t.valueOf()))
      );
    yAxis = d3.axisLeft(y);

    gxAxis = svgGroup
      .append("g")
      .attr("transform", `translate(0, ${height})`)
      .call(xAxis);

    gyAxis = svgGroup.append("g").call(yAxis);

    zoom = d3
      .zoom()
      .filter((e) => !e.shiftKey)
      .scaleExtent([1, getPixelSizeNs()])
      .on("zoom", (event) => {
        transform = event.transform;
      });

    refreshZoom();

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    container.call(zoom as any);

    brush = container
      .select("svg")
      .append("g")
      .on("contextmenu", (e) => {
        e.preventDefault();
      });

    var brushFunction = d3
      .brushX()
      .filter((e) => e.shiftKey)
      .extent([
        [margin.left + 1, margin.top],
        [width, height + margin.top - 1],
      ])
      .on("end", (e: d3.D3BrushEvent<number>) => {
        const scaleX = transform.rescaleX(x);
        const selection = e.selection as [number, number];
        brushStart = scaleX.invert(selection[0]).valueOf();
        brushEnd = scaleX.invert(selection[1]).valueOf();
      });

    brush.call(brushFunction);
  }

  function updateChart() {
    if (!container) {
      return;
    }

    if (brush) {
      brush.call(d3.brush().clear);
      brushStart = -Infinity;
      brushEnd = Infinity;
    }

    updateLod();

    var startTime = performance.now();

    refreshZoom();

    container.select("svg").attr("height", outerHeight).attr("width", width);

    container
      .select("canvas")
      .attr("height", height)
      .attr("width", width - margin.left);

    x.range([0, width]);

    const yMax = d3.max(
      points
        .map((p) => p.points)
        .flatMap((p) => d3.max(p.map((point) => point.value)) ?? 0)
    );

    const yMin = d3.min(
      points
        .map((p) => p.points)
        .flatMap((p) => d3.min(p.map((point) => point.value)) ?? 0)
    );

    y.range([height, 0]).domain([yMin ?? 0, yMax ?? 0]);

    draw();

    updateTime = Math.floor(performance.now() - startTime);
  }

  function draw() {
    const scaleX = transform.rescaleX(x);

    context.fillStyle = "rgba(0, 0, 0, 0)";
    context.fillRect(0, 0, width, height);

    var line = d3
      .line()
      .x((d) => scaleX(d[0]))
      .y((d) => y(d[1]))
      .context(context);

    points.forEach((data, i) => {
      context.beginPath();
      line(
        data.points.map((newPoints) => [newPoints.tickOffset, newPoints.value])
      );
      const color = Math.abs(hashString(data.name)) % 10;
      context.strokeStyle = d3.schemeCategory10[color];
      context.lineWidth = 0.33;
      context.stroke();
    });

    gxAxis.call(xAxis.scale(scaleX));
    gyAxis.call(yAxis.scale(y));
  }
</script>

{#if !loading}
  <MetricSelection
    metrics={$metricStore}
    on:metric-switched={(e) => {
      metricStreamer.updateFromSelectionState(e.detail.metric);
    }}
  />
{/if}
<div bind:clientWidth={mainWidth}>
  <div id="metric-canvas" style="position:relative" />
  {#if loading}
    <div>Loading...</div>
  {:else}
    <div class="grid grid-cols-3">
      <div>
        <div><span class="font-bold">Width</span>: {width}</div>
        <div><span class="font-bold"> Main Width</span>: {mainWidth}</div>
        <br />
        <div>
          <span class="font-bold">Update Time</span>: {updateTime} ms
        </div>
        <div>
          <span class="font-bold">Transform</span>
          <span class="font-bold">X</span>
          {transform.x.toFixed(2)}
          <span class="font-bold">Y</span>
          {transform.y.toFixed(2)}
        </div>
        <ul>
          <li>
            <span class="font-bold">Zoom</span>
            {transform.k}
          </li>
          <li>
            <span class="font-bold">Lod</span>
            {lod}
          </li>
          <li>
            <span class="font-bold">Pixel size</span>
            {formatExecutionTime(pixelSizeNs / 1_000_000)}
          </li>
          <li>
            <span class="font-bold">Delta Ms</span>
            {formatExecutionTime(deltaMs)}
          </li>
          <br />
          <li>
            <span class="font-bold">Min</span>
            {totalMinTick}
          </li>
          <li>
            <span class="font-bold">Current Min</span>
            {currentMinTick}
          </li>
          <li>
            <span class="font-bold">Max</span>
            {totalMaxTick}
          </li>
          <li>
            <span class="font-bold">Current Max</span>
            {currentMaxTick}
          </li>
          <li>
            <span class="font-bold">BrushStart</span>
            {brushStart}
            /
            <span class="font-bold">BrushEnd</span>
            {brushEnd}
          </li>
        </ul>
      </div>
      <div style="font-size:0.8rem">
        {#if metricStreamer}
          <ul>
            {#each $metricStore as ms}
              {#if ms.enabled}
                <li>
                  {ms.name} (unit: {ms.unit})<br />
                  {ms.minTick} _ {ms.maxTick} ({formatExecutionTime(
                    metricStreamer.getTickOffsetMs(ms.maxTick) -
                      metricStreamer.getTickOffsetMs(ms.minTick)
                  )})<br />
                  {#each Array.from(ms.getViewportBlocks(currentMinTick, currentMaxTick)) as b}
                    <div style="font-size:0.7rem">
                      {b.blockId}
                      {b.minTick.toFixed(0)}
                      {b.maxTick.toFixed(0)} ({formatExecutionTime(
                        metricStreamer.getTickOffsetMs(b.maxTick) -
                          metricStreamer.getTickOffsetMs(b.minTick)
                      )}) ({Array.from(
                        b.getPoints(currentMinTick, currentMaxTick, lod)
                      ).length})
                    </div>
                  {/each}
                </li>
              {/if}
            {/each}
          </ul>
        {/if}
      </div>
      <div>
        <TimeRangeDetails timeRange={[brushStart, brushEnd]} processId={id} />
      </div>
    </div>
  {/if}
</div>
