<script lang="ts">
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { onMount } from "svelte";
  export let stateStore: TimelineStateStore;
  import * as d3 from "d3";
  import { threadItemLength } from "@/lib/Timeline/TimelineValues";
  import { formatExecutionTime } from "@/lib/format";

  const height = 20;
  const padding = 4;
  const tickSize = 150;

  let svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;
  let x = d3.scaleLinear();
  let ticks: number[] = [];
  let el: HTMLElement;

  $: if ($stateStore && svg) {
    const tickCount = Math.ceil($stateStore.canvasWidth / tickSize);
    svg = svg.attr("width", $stateStore.canvasWidth);
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
      .attr("width", el.offsetWidth)
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
    color: gray;
  }
</style>
