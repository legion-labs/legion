<script lang="ts">
  import * as d3 from "d3";
  import { onMount } from "svelte";

  import { formatExecutionTime } from "@/lib/format";

  import type { TimelineStateStore } from "../Lib/TimelineStateStore";
  import { threadItemLength } from "../Values/TimelineValues";

  export let stateStore: TimelineStateStore;

  const height = 20;
  const padding = 4;
  const tickSize = 150;

  let svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;
  let x = d3.scaleLinear();
  let ticks: number[] = [];
  let el: HTMLElement;

  $: if ($stateStore && svg) {
    const tickCount = Math.ceil($stateStore.canvasWidth / tickSize);
    svg = svg.attr("width", Math.max(0, $stateStore.canvasWidth));
    x.range([0, $stateStore.canvasWidth]).domain($stateStore.getViewRange());
    ticks.length = 0;
    svg.call(
      d3
        .axisBottom(x)
        .ticks(tickCount)
        .tickFormat((x, i) => formatTick(x, i))
    );
  }

  onMount(() => {
    svg = d3
      .select(el)
      .append("svg")
      .attr("transform", `translate(${threadItemLength + padding},0)`)
      .attr("width", Math.max(0, el.offsetWidth))
      .attr("height", height);
  });

  function formatTick(x: d3.NumberValue, i: number) {
    ticks[i] = x.valueOf();
    const value = (i > 0 ? ticks[i] - ticks[i - 1] : 0) * i;
    return `${value > 0 ? "+" : ""}${formatExecutionTime(value, 1)}`;
  }
</script>

<div bind:this={el} class="axis" />

<style lang="postcss">
  .axis {
    user-select: none;
    color: #bababa;
  }
</style>
