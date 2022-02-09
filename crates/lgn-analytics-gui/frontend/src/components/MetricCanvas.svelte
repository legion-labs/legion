<script lang="ts">
  import { client } from "@/lib/client";
  import { formatExecutionTime } from "@/lib/format";
  import { Point } from "@/lib/point";
  import * as d3 from "d3";
  import { onDestroy, onMount } from "svelte";
  import { Unsubscriber, Writable } from "svelte/store";
  import { MetricState, MetricStreamer } from "./MetricStreamer";
  import TimeRangeDetails from "./TimeRangeDetails.svelte";
  export let id: string;

  let metricStreamer: MetricStreamer;
  let metricStore: Writable<MetricState[]>;
  const margin = { top: 20, right: 50, bottom: 60, left: 70 };

  const outerHeight = 600;
  const height = outerHeight - margin.top - margin.bottom;

  let mainWidth: number = 0;
  $: width = mainWidth - margin.left - margin.right;

  let totalMinMs = -Infinity;
  let totalMaxMs = Infinity;
  let currentMinMs = -Infinity;
  let currentMaxMs = Infinity;
  let brushStart = -Infinity;
  let brushEnd = Infinity;
  let points: Point[][] = [];
  let loading = true;
  let updateTime: number;
  let lod: number;
  let deltaMs: number;
  let pixelSizeNs: number;

  let x: d3.ScaleTime<number, number, never>;
  let y: d3.ScaleLinear<number, number, never>;

  let gxAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let gyAxis: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let xAxis: d3.Axis<d3.NumberValue>;
  let yAxis: d3.Axis<d3.NumberValue>;

  let container: d3.Selection<d3.BaseType, unknown, HTMLElement, any>;
  let brush: d3.Selection<SVGGElement, unknown, HTMLElement, any>;
  let context: CanvasRenderingContext2D;
  let transform: d3.ZoomTransform = d3.zoomIdentity;
  let canvas: HTMLCanvasElement;
  let pointSubscription: Unsubscriber | undefined;

  $: {
    if (mainWidth && transform) {
      updateChart();
    }
  }

  const getDeltaMs = () => currentMaxMs - currentMinMs;
  const getPixelSizeNs = () => (getDeltaMs() * 1_000_000) / width;
  const getLod = () =>
    Math.max(0, Math.floor(Math.log(getPixelSizeNs()) / Math.log(100)));

  onMount(async () => {
    await fetchMetricsAsync().then(() => (loading = false));
    createChart();
    updateChart();
  });

  onDestroy(() => {
    document.getElementsByClassName("canvas")[0].replaceChildren();
    if (pointSubscription) {
      pointSubscription();
    }
  });

  function updateLod() {
    if (x) {
      const scaleX = transform.rescaleX(x);
      currentMinMs = scaleX.domain()[0].valueOf();
      currentMaxMs = scaleX.domain()[1].valueOf();
    }
    deltaMs = getDeltaMs();
    pixelSizeNs = getPixelSizeNs();
    lod = getLod();
    metricStreamer!.tick(lod, currentMinMs, currentMaxMs);
  }

  async function fetchMetricsAsync() {
    const reply = await client.list_process_metrics({ processId: id });
    totalMinMs = currentMinMs = reply.minTimeMs;
    totalMaxMs = currentMaxMs = reply.maxTimeMs;
    metricStreamer = new MetricStreamer(id, getLod(), totalMinMs, totalMaxMs);
    metricStore = metricStreamer.metricStore;
    await metricStreamer.initializeAsync();
    pointSubscription = metricStore.subscribe((metricState) => {
      points = metricState.filter((m) => m.enabled).map((m) => m.points);
      updateChart();
    });
    updateLod();
  }

  function createChart() {
    container = d3.select(".canvas");

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

    context = canvas.getContext("2d")!;

    x = d3.scaleTime().domain([totalMinMs, totalMaxMs]).nice();
    y = d3.scaleLinear().nice();

    xAxis = d3
      .axisBottom(x)
      .tickFormat((d) => formatExecutionTime((d as Date).valueOf()));
    yAxis = d3.axisLeft(y);

    gxAxis = svgGroup
      .append("g")
      .attr("transform", `translate(0, ${height})`)
      .call(xAxis);

    gyAxis = svgGroup.append("g").call(yAxis);

    const zoom = d3
      .zoom()
      .filter((e) => !e.shiftKey)
      .scaleExtent([1, getPixelSizeNs()])
      .translateExtent([[0, 0], getTranslateExtent()])
      .on("zoom", (event) => {
        transform = event.transform;
      });

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

    container.select("svg").attr("height", outerHeight).attr("width", width);

    container
      .select("canvas")
      .attr("height", height)
      .attr("width", width - margin.left);

    x.range([0, width]);

    const yMax = d3.max(
      points.flatMap(
        (newPoints) =>
          d3.max(
            newPoints
              .filter(
                (newPoints) =>
                  newPoints.time >= currentMinMs &&
                  newPoints.time <= currentMaxMs
              )
              .map((newPoints) => newPoints.value)
          ) ?? 0
      )
    );

    y.range([height, 0]).domain([0, yMax ?? 0]);

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

    points.forEach((points, i) => {
      context.beginPath();
      line(points.map((newPoints) => [newPoints.time, newPoints.value]));
      context.strokeStyle = d3.schemeCategory10[i];
      context.lineWidth = 0.33;
      context.stroke();
    });

    gxAxis.call(xAxis.scale(scaleX));
    gyAxis.call(yAxis.scale(y));
  }

  function getTranslateExtent(): [number, number] {
    return [mainWidth, outerHeight];
  }
</script>

<div bind:clientWidth={mainWidth}>
  <div class="canvas" />
  {#if loading}
    <div>Loading...</div>
  {:else}
    <div class="grid grid-cols-2">
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
            {totalMinMs.toFixed(2)}
          </li>
          <li>
            <span class="font-bold">Current Min</span>
            {currentMinMs.toFixed(2)}
          </li>
          <li>
            <span class="font-bold">Max</span>
            {totalMaxMs.toFixed(2)}
          </li>
          <li>
            <span class="font-bold">Current Max</span>
            {currentMaxMs.toFixed(2)}
          </li>
          <li>
            <span class="font-bold">BrushStart</span>
            {brushStart}
            /
            <span class="font-bold">BrushEnd</span>
            {brushEnd}
          </li>
        </ul>
        <br />
        {#if metricStreamer}
          <ul>
            {#each $metricStore as ms}
              <li>
                <input
                  type="checkbox"
                  id={ms.metricDesc.name + "_select"}
                  checked={ms.enabled}
                  on:click={(e) => metricStreamer.switchMetric(ms, e)}
                />
                {ms.metricDesc.name} (unit: {ms.metricDesc.unit})
              </li>
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
