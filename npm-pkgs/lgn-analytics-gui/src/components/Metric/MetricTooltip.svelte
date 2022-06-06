<script lang="ts">
  import type { D3ZoomEvent } from "d3";
  import { getContext } from "svelte";

  import { getMetricColor } from "./Lib/MetricColor";
  import type { MetricPoint } from "./Lib/MetricPoint";
  import type { MetricState } from "./Lib/MetricState";
  import MetricTooltipItem from "./MetricTooltipItem.svelte";

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let zoomEvent: D3ZoomEvent<HTMLCanvasElement, any>;
  export let xScale: d3.ScaleLinear<number, number, never>;
  export let lod: number;
  export let viewRange: [number, number];

  const margin = 15;

  const selectedMetricStore = getContext("selected-metrics-store");

  let displayed = false;
  let xValue: number;
  let yValue: number;
  let side: boolean;
  let width: number;
  let displayInternal: boolean;
  let values: {
    metric: MetricState;
    value: MetricPoint | null;
    color: string;
  }[];

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

  $: time = xScale.invert(xValue);

  $: side = xValue < (2 * width) / 3;

  $: {
    if (zoomEvent?.sourceEvent) {
      xValue = zoomEvent.sourceEvent.offsetX;
      yValue = zoomEvent.sourceEvent.offsetY;
    }
  }

  $: {
    if (xValue) {
      values = $selectedMetricStore.map((metric, index) => {
        return {
          metric,
          value: getClosestValue(metric, {
            time,
            min: viewRange[0],
            max: viewRange[1],
            lod,
          }),
          color: getMetricColor(index),
        };
      });

      displayInternal = values.some((v) => v.value);
    }
  }

  $: style = side
    ? `top:${yValue}px;left:${xValue + margin}px`
    : `top:${yValue}px;right:${width - xValue + margin}px`;

  function getClosestValue(
    metric: MetricState,
    options: { time: number; min: number; max: number; lod: number }
  ) {
    const m = $selectedMetricStore.filter((m) => m.name === metric.name)[0];

    if (m) {
      return m.getClosestValue(options);
    }

    return null;
  }
</script>

<div bind:clientWidth={width}>
  {#if displayed && values && displayInternal}
    <div
      class="shadow-md surface absolute pointer-events-none text-sm p-2 flex flex-col z-20"
      {style}
    >
      {#each values as metric (metric.metric.name)}
        {#if metric.value !== null}
          <MetricTooltipItem {...metric} />
        {/if}
      {/each}
    </div>
  {/if}
</div>
