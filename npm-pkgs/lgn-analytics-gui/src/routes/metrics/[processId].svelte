<script lang="ts">
  import { page } from "$app/stores";
  import * as d3 from "d3";
  import type { D3ZoomEvent } from "d3";
  import { onDestroy, onMount } from "svelte";
  import { setContext } from "svelte";
  import { getContext } from "svelte";
  import type { Unsubscriber } from "svelte/store";

  import { MetricAxisCollection } from "@/components/Metric/Lib/MetricAxisCollection";
  import { getMetricColor } from "@/components/Metric/Lib/MetricColor";
  import type { MetricSlice } from "@/components/Metric/Lib/MetricSlice";
  import type { MetricState } from "@/components/Metric/Lib/MetricState";
  import {
    getLastUsedMetricsStore,
    getMetricConfigStore,
    getMetricNames,
    getMetricStore,
    getRecentlyUsedMetricsStore,
  } from "@/components/Metric/Lib/MetricStore";
  import { MetricStreamer } from "@/components/Metric/Lib/MetricStreamer";
  import MetricDebugDisplay from "@/components/Metric/MetricDebugDisplay.svelte";
  import MetricLegendGroup from "@/components/Metric/MetricLegendGroup.svelte";
  import MetricSelection from "@/components/Metric/MetricSelection.svelte";
  import MetricTooltip from "@/components/Metric/MetricTooltip.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import TimeRangeDetails from "@/components/Misc/TimeRangeDetails.svelte";
  import { formatExecutionTime } from "@/lib/format";
  import { getLodFromPixelSizeNs } from "@/lib/lod";

  const processId = $page.params.processId;

  let metricStreamer: MetricStreamer;
  let axisCollection: MetricAxisCollection;

  const lastUsedMetricsStore = getLastUsedMetricsStore();

  const metricConfigStore = getMetricConfigStore();

  const metricStore = getMetricStore(lastUsedMetricsStore, metricConfigStore);

  const recentlyUsedMetricsStore = getRecentlyUsedMetricsStore(
    metricStore,
    metricConfigStore
  );

  const metricNames = getMetricNames();

  setContext("metrics-store", metricStore);

  setContext("metrics-config-store", metricConfigStore);

  setContext("recently-used-metrics-store", recentlyUsedMetricsStore);

  const defaultLineWidth = 1;
  const margin = { top: 20, right: 50, bottom: 40, left: 70 };
  const outerHeight = 600;
  const height = outerHeight - margin.top - margin.bottom;

  const client = getContext("http-client");
  const debug = getContext("debug");

  let mainWidth = 0;
  $: width = mainWidth - margin.left - margin.right;

  let metricTooltip: MetricTooltip;
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
  // let bestY: d3.ScaleLinear<number, number, never>;

  let brushFunction: d3.BrushBehavior<unknown>;
  let svgGroup: d3.Selection<SVGGElement, unknown, HTMLElement, unknown>;
  let gxAxis: d3.Selection<SVGGElement, unknown, HTMLElement, unknown>;
  // let gyAxis: d3.Selection<SVGGElement, unknown, HTMLElement, unknown>;
  let container: d3.Selection<d3.BaseType, unknown, HTMLElement, unknown>;
  let zoomEvent: D3ZoomEvent<HTMLCanvasElement, unknown>;
  let brushSvg: d3.Selection<SVGGElement, unknown, HTMLElement, unknown>;

  let xAxis: d3.Axis<d3.NumberValue>;
  // let yAxis: d3.Axis<d3.NumberValue>;
  let zoom: d3.ZoomBehavior<Element, unknown>;

  let context: CanvasRenderingContext2D;
  let transform: d3.ZoomTransform = d3.zoomIdentity;
  let canvas: HTMLCanvasElement;
  let pointSubscription: Unsubscriber | undefined;

  $: if (!loading) {
    if (transform) {
      updateLod();
      updatePoints($metricStore);
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
    axisCollection = new MetricAxisCollection();

    await fetchMetrics();

    createChart();
    updateLod();
    updatePoints($metricStore);
    updateChart();
    tick();

    loading = false;
  });

  onDestroy(() => {
    canvas?.replaceChildren();
    pointSubscription?.();
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

  async function fetchMetrics() {
    metricStreamer = new MetricStreamer(
      client,
      processId,
      metricStore,
      lastUsedMetricsStore,
      metricNames
    );
    await metricStreamer.initialize();

    totalMinMs = currentMinMs = metricStreamer.currentMinMs;
    totalMaxMs = currentMaxMs = metricStreamer.currentMaxMs;

    pointSubscription = metricStore.subscribe((metricStates) => {
      updateInternal(metricStates);
    });
  }

  function updateInternal(states: MetricState[]) {
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

    // Forwards click event to the document's body
    // so that the clickOutside action can work properly
    container.on("click", () =>
      document.body.dispatchEvent(
        new MouseEvent("mouseup", {
          view: window,
          bubbles: true,
          cancelable: true,
        })
      )
    );

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

    svgGroup.on("mouseover", (_e) => {
      metricTooltip.enable();
    });

    svgGroup.on("mouseout", (_e) => {
      metricTooltip.hide();
    });

    canvas = canvasChart.node() as HTMLCanvasElement;

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    context = canvas.getContext("2d")!;

    x = d3.scaleLinear().domain([totalMinMs, totalMaxMs]);

    xAxis = d3
      .axisBottom(x)
      .tickFormat((d) => formatExecutionTime(d.valueOf()));

    // bestY = axisCollection.getBestAxisScale([height, 0], $metricStore);
    // yAxis = d3.axisLeft(bestY);

    gxAxis = svgGroup
      .append("g")
      .style("user-select", "none")
      .attr("transform", `translate(0, ${height})`)
      .call(xAxis);

    // Remove y axis for now
    // gyAxis = svgGroup.append("g").style("user-select", "none").call(yAxis);

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
    // bestY = axisCollection.getBestAxisScale([height, 0], $metricStore);
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
    let lineWidth = +import.meta.env
      .VITE_LEGION_ANALYTICS_METRICS_LINE_THICKNESS;

    if (isNaN(lineWidth) || lineWidth <= 0) {
      lineWidth = defaultLineWidth;
    }

    const scaleX = transform.rescaleX(x);
    for (const metric of points) {
      const color = (context.strokeStyle = getMetricColor(metric.name));
      const scaleY = axisCollection.getAxisScale(metric.unit, [height, 0]);
      var line = getLine(scaleX, scaleY);
      context.beginPath();
      line(metric.points.map((p) => [p.time, p.value]));
      context.strokeStyle = color;
      context.lineWidth = lineWidth;
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
    // Remove y axis for now
    // gyAxis.call(yAxis.scale(bestY));
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

<Layout>
  <div class="metrics" slot="content">
    {#if !loading}
      <MetricSelection />
      <MetricTooltip
        bind:this={metricTooltip}
        {metricStore}
        xScale={transform.rescaleX(x)}
        leftMargin={margin.left}
        {zoomEvent}
      />
    {/if}

    <div bind:clientWidth={mainWidth}>
      <div id="metric-canvas" class="relative" />

      {#if !loading}
        <div style="padding-left:{margin.left}px">
          <MetricLegendGroup {metricStore} />
        </div>
        <div>
          <TimeRangeDetails timeRange={[brushStart, brushEnd]} {processId} />
        </div>
        {#if $debug}
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
      {/if}
    </div>
  </div>
</Layout>

<style lang="postcss">
  .metrics {
    @apply pt-4 px-2;
  }
</style>
