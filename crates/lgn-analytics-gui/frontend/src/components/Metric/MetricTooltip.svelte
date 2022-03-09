<script lang="ts">
  import { Point } from "@/lib/Metric/MetricPoint";
  import { selectionStore } from "@/lib/Metric/MetricSelectionStore";
  import { MetricStreamer } from "@/lib/Metric/MetricStreamer";
  import { D3ZoomEvent } from "d3";
  import { get } from "svelte/store";
  import { MetricSelectionState } from "./MetricSelectionState";
  import MetricTooltipItem from "./MetricTooltipItem.svelte";
  export let xScale: d3.ScaleLinear<number, number, never>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let zoomEvent: D3ZoomEvent<HTMLCanvasElement, any>;
  export let metricStreamer: MetricStreamer;
  export let leftMargin: number;
  const margin = 15;
  let displayed = false;
  let xValue: number;
  let yValue: number;
  let side: boolean;
  let width: number;
  let displayInternal: boolean;
  let values: { metric: MetricSelectionState; value: Point | null }[];

  export function enable() {
    displayed = true;
  }

  export function show(x: number, y: number) {
    enable();
    xValue = x;
    yValue = y;
  }

  export function hide() {
    displayed = false;
  }

  $: time = xScale.invert(xValue - leftMargin);
  $: side = xValue < (2 * width) / 3;
  $: {
    if (zoomEvent?.sourceEvent) {
      xValue = zoomEvent.sourceEvent.offsetX;
      yValue = zoomEvent.sourceEvent.offsetY;
    }
  }
  $: {
    if (xValue) {
      values = get(selectionStore)
        .filter((m) => !m.hidden && m.selected)
        .map((metric) => {
          return {
            metric,
            value: getClosestValue(metric, time),
          };
        });
      displayInternal = values.some((v) => v.value);
    }
  }
  $: style = side
    ? `top:${yValue}px;left:${xValue + margin}px`
    : `top:${yValue}px;right:${width - xValue + margin}px`;

  function getClosestValue(metric: MetricSelectionState, time: number) {
    if (!metricStreamer?.metricStore) {
      return null;
    }
    const store = metricStreamer.metricStore;
    const m = get(store).filter((m) => m.name === metric.name)[0];
    if (m) {
      return m.getClosestValue(time);
    }
    return null;
  }
</script>

<div bind:clientWidth={width}>
  {#if displayed && values && displayInternal}
    <div class="main text-sm p-2 flex flex-col gap-1" {style}>
      {#each values as metric (metric.metric.name)}
        {#if metric.metric.selected && !metric.metric.hidden && metric.value?.value}
          <MetricTooltipItem metric={metric.metric} value={metric.value} />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style lang="postcss">
  .main {
    @apply shadow-md bg-gray-50;
    position: absolute;
    z-index: 20;
    pointer-events: none;
  }
</style>
