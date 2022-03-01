<script lang="ts">
  import { makeGrpcClient } from "@/lib/client";
  import { formatExecutionTime } from "@/lib/format";
  import { getLodFromPixelSizeNs } from "@/lib/lod";
  import { MetricAxisCollection } from "@/lib/Metric/MetricAxisCollection";
  import { getMetricColor } from "@/lib/Metric/MetricColor";
  import { selectionStore } from "@/lib/Metric/MetricSelectionStore";
  import { MetricSlice } from "@/lib/Metric/MetricSlice";
  import { MetricState } from "@/lib/Metric/MetricState";
  import { MetricStreamer } from "@/lib/Metric/MetricStreamer";
  import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
  import log from "@lgn/web-client/src/lib/log";
  import * as d3 from "d3";
  import { D3ZoomEvent } from "d3";
  import { onDestroy, onMount } from "svelte";
  import { get, Unsubscriber, Writable } from "svelte/store";
  import MetricDebugDisplay from "./Metric/MetricDebugDisplay.svelte";
  import MetricLegendGroup from "./Metric/MetricLegendGroup.svelte";
  import MetricSelection from "./Metric/MetricSelection.svelte";
  import MetricTooltip from "./Metric/MetricTooltip.svelte";
  export let id: string;

  let metricStreamer: MetricStreamer;
  let axisCollection: MetricAxisCollection;
  let metricStore: Writable<MetricState[]>;

  const margin = { top: 20, right: 50, bottom: 40, left: 70 };

  const outerHeight = 600;
  const height = outerHeight - margin.top - margin.bottom;

  let mainWidth = 0;
  $: width = mainWidth - margin.left - margin.right;

  let metricTooltip: MetricTooltip;
  let client: PerformanceAnalyticsClientImpl | null = null;
  let totalMinMs = -Infinity;
  let totalMaxMs = Infinity;
  let currentMinMs = -Infinity;
  let currentMaxMs = Infinity;
  let brushStart = NaN;
  let brushEnd = NaN;
  let points: MetricSlice[];
  let loading = true;
  let updateTime: number;
  let lod: number;
  let deltaMs: number;
  let pixelSizeNs: number;
  let x: d3.ScaleLinear<number, number, never>;
  let bestY: d3.ScaleLinear<number, number, never>;

  let brushFunction: d3.BrushBehavior<unknown>;
  /* eslint-disable @typescript-eslint/no-explicit-any */
  let svgGroup: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let gxAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let gyAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let container: d3.Selection<d3.BaseType, unknown, HTMLElement, any>;
  let zoomEvent: D3ZoomEvent<HTMLCanvasElement, any>;
  let brushSvg: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  /* eslint-enable @typescript-eslint/no-explicit-any */

  let xAxis: d3.Axis<d3.NumberValue>;
  let yAxis: d3.Axis<d3.NumberValue>;
  let zoom: d3.ZoomBehavior<Element, unknown>;

  let context: CanvasRenderingContext2D;
  let transform: d3.ZoomTransform = d3.zoomIdentity;
  let canvas: HTMLCanvasElement;
  let pointSubscription: Unsubscriber | undefined;
  let selectionSubsription: Unsubscriber | undefined;

  $: if (!loading) {
    if (transform) {
      updateLod();
      updatePoints(get(metricStore));
      updateChart();
      tick();
    }
  }

  $: {
    if (mainWidth) {
      transform = transform;
    }
  }

  const getDeltaMs = () => currentMaxMs - currentMinMs;
  const getPixelSizeNs = () => (getDeltaMs() * 1_000_000) / width;

  onMount(async () => {
    client = await makeGrpcClient();
    axisCollection = new MetricAxisCollection();
    await fetchMetricsAsync().then(() => {
      createChart();
      updateLod();
      updatePoints(get(metricStore));
      updateChart();
      tick();
      loading = false;
    });
  });

  onDestroy(() => {
    if (canvas) {
      canvas.replaceChildren();
    }
    if (pointSubscription) {
      pointSubscription();
    }
    if (selectionSubsription) {
      selectionSubsription();
    }
  });

  function updateLod() {
    deltaMs = getDeltaMs();
    pixelSizeNs = getPixelSizeNs();
    lod = getLodFromPixelSizeNs(pixelSizeNs);
    if (x) {
      x.range([0, width]);
      const scaleX = transform.rescaleX(x);
      currentMinMs = scaleX.domain()[0].valueOf();
      currentMaxMs = scaleX.domain()[1].valueOf();
    }
  }

  function tick() {
    metricStreamer?.tick(lod, currentMinMs, currentMaxMs);
  }

  async function fetchMetricsAsync() {
    if (!client) {
      log.error("no client in fetchMetricsAsync");
      return;
    }

    metricStreamer = new MetricStreamer(id);
    metricStore = metricStreamer.metricStore;
    await metricStreamer.initializeAsync();

    totalMinMs = currentMinMs = metricStreamer.currentMinMs;
    totalMaxMs = currentMaxMs = metricStreamer.currentMaxMs;

    selectionSubsription = selectionStore.subscribe(() => {
      update(get(metricStore));
    });

    pointSubscription = metricStore.subscribe((metricStates) => {
      update(metricStates);
    });
  }

  function update(states: MetricState[]) {
    updateLod();
    updatePoints(states);
    updateChart();
    tick();
  }

  function updatePoints(states: MetricState[]) {
    if (!states) {
      return;
    }

    points = states
      .filter((m) => m.canBeDisplayed())
      .map((m) => {
        return {
          points: Array.from(
            m.getViewportPoints(currentMinMs, currentMaxMs, lod, true)
          ),
          name: m.name,
          unit: m.unit,
        };
      });

    axisCollection.update(points);
  }

  function refreshZoom() {
    const extent = [width, outerHeight] as [number, number];
    const origin = [0, 0] as [number, number];
    zoom.translateExtent([origin, extent]);
    zoom.extent([origin, extent]);
  }

  function createChart() {
    container = d3.select("#metric-canvas");

    svgGroup = container
      .append("svg")
      .append("g")
      .attr("transform", `translate(${margin.left}, ${margin.top})`);

    let fo = svgGroup
      .append("foreignObject")
      .attr("x", 0)
      .attr("y", 0)
      .attr("width", width)
      .attr("height", height);

    var foBody = fo
      .append("xhtml:body")
      .style("width", `${width}px`)
      .style("height", `${height}px`);

    const canvasChart = foBody.append("canvas");

    svgGroup.on("mousemove", (e: MouseEvent) => {
      metricTooltip.show(e.offsetX, e.offsetY);
    });

    svgGroup.on("mouseover", (e) => {
      metricTooltip.enable();
    });

    svgGroup.on("mouseout", (e) => {
      metricTooltip.hide();
    });

    canvas = canvasChart.node() as HTMLCanvasElement;

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    context = canvas.getContext("2d")!;

    x = d3.scaleLinear().domain([totalMinMs, totalMaxMs]);

    xAxis = d3
      .axisBottom(x)
      .tickFormat((d) => formatExecutionTime(d.valueOf()));

    bestY = axisCollection.getBestAxisScale([height, 0]);
    yAxis = d3.axisLeft(bestY);

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
        zoomEvent = event;
        transform = event.transform;
        if (brushEnd && brushStart) {
          const scaleX = transform.rescaleX(x);
          const start = scaleX(brushStart).valueOf();
          const end = scaleX(brushEnd).valueOf();
          brushSvg.call(brushFunction.move, [
            Math.max(0, start),
            Math.max(0, end),
          ]);
        }
      });

    refreshZoom();

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    svgGroup.call(zoom as any);

    brushFunction = d3
      .brushX()
      .filter((e) => e.shiftKey)
      .extent([
        [1, 0],
        [width - margin.left, height - 1],
      ])
      .on("end", (e: d3.D3BrushEvent<number>) => {
        const scaleX = transform.rescaleX(x);
        const selection = e.selection as [number, number];
        brushStart = scaleX.invert(selection[0]).valueOf();
        brushEnd = scaleX.invert(selection[1]).valueOf();
      });

    brushSvg = svgGroup.append("g");

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    brushSvg.call(brushFunction as any);
  }

  function updateChartWidth() {
    if (container) {
      container.select("svg").attr("height", outerHeight).attr("width", width);
      container
        .select("canvas")
        .attr("height", height)
        .attr("width", width - margin.left);
    }
  }

  function updateChart() {
    if (!container) {
      return;
    }

    var startTime = performance.now();
    x.range([0, width]);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    svgGroup.call(zoom as any);
    refreshZoom();
    updateChartWidth();
    bestY = axisCollection.getBestAxisScale([height, 0]);
    draw();
    updateTime = Math.floor(performance.now() - startTime);
  }

  function getLine(
    x: d3.ScaleLinear<number, number, never>,
    y: d3.ScaleLinear<number, number, never>
  ): d3.Line<[number, number]> {
    return d3
      .line()
      .x((d) => x(d[0]))
      .y((d) => y(d[1]))
      .context(context);
  }

  function draw() {
    const scaleX = transform.rescaleX(x);
    for (const metric of points) {
      const color = (context.strokeStyle = getMetricColor(metric.name));
      const scaleY = axisCollection.getAxisScale(metric.unit, [height, 0]);
      var line = getLine(scaleX, scaleY);
      context.beginPath();
      line(metric.points.map((p) => [p.time, p.value]));
      context.strokeStyle = color;
      context.lineWidth = 0.33;
      context.stroke();
      if (lod <= 3) {
        for (const point of metric.points) {
          context.beginPath();
          context.arc(
            scaleX(point.time),
            scaleY(point.value),
            1,
            0,
            2 * Math.PI
          );
          context.fillStyle = color;
          context.fill();
        }
      }
    }

    gxAxis.call(xAxis.scale(scaleX));
    gyAxis.call(yAxis.scale(bestY));
  }

  function handleKeydown(event: KeyboardEvent) {
    if (brushStart && brushEnd && event.code == "Escape") {
      brushSvg.call(d3.brush().clear);
      brushStart = NaN;
      brushEnd = NaN;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if !loading}
  <MetricSelection />
  <MetricTooltip
    bind:this={metricTooltip}
    xScale={transform.rescaleX(x)}
    leftMargin={margin.left}
    {zoomEvent}
    {metricStreamer}
  />
{/if}
<div bind:clientWidth={mainWidth}>
  <div id="metric-canvas" style="position:relative" />
  {#if loading}
    <div>Loading...</div>
  {:else}
    <div style="padding-left:{margin.left}px">
      <MetricLegendGroup />
    </div>
    <div style="display:inherit;padding-top:40px">
      <MetricDebugDisplay
        {width}
        {mainWidth}
        {transform}
        {updateTime}
        {metricStreamer}
        {lod}
        {pixelSizeNs}
        {deltaMs}
        {id}
        {totalMinMs}
        {currentMinMs}
        {totalMaxMs}
        {currentMaxMs}
        {brushStart}
        {brushEnd}
        {metricStore}
      />
    </div>
  {/if}
</div>
