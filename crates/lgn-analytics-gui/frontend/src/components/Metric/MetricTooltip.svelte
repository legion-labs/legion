<script lang="ts">
  import { selectionStore } from "@/lib/Metric/MetricSelectionStore";
  import { MetricStreamer } from "@/lib/Metric/MetricStreamer";
  import { D3ZoomEvent } from "d3";
  import { get } from "svelte/store";
  import { MetricSelectionState } from "./MetricSelectionState";
  import MetricTooltipItem from "./MetricTooltipItem.svelte";
  export let xScale: d3.ScaleLinear<number, number, never>;
  export let zoomEvent: D3ZoomEvent<HTMLCanvasElement, any>;
  export let metricStreamer: MetricStreamer;
  const margin = 15;
  let displayed = false;
  let xValue: number;
  let yValue: number;
  let side: boolean;
  let width: number;

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

  $: xValue = zoomEvent?.sourceEvent?.offsetX ?? 0;
  $: yValue = zoomEvent?.sourceEvent?.offsetY ?? 0;
  $: time = xScale.invert(xValue);
  $: side = xValue < (2 * width) / 3;
  $: values = get(selectionStore).map((metric) => {
    return {
      metric,
      value: getClosestValue(metric, time),
    };
  });

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
  {#if displayed}
    <div class="main text-sm flex flex-col gap-1 p-2" {style}>
      {#each values as metric}
        {#if metric.metric.selected && !metric.metric.hidden}
          {#if metric.value?.value}
            <MetricTooltipItem metric={metric.metric} value={metric.value} />
          {/if}
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .main {
    @apply shadow-md bg-gray-50;
    position: absolute;
    z-index: 20;
    pointer-events: none;
  }
</style>
