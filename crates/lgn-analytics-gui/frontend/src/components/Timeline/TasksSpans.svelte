<script lang="ts">
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { Process } from "@lgn/proto-telemetry/dist/process";
  import { ProcessAsyncData } from "@/lib/Timeline/ProcessAsyncData";
  import { DrawSelectedRange } from "@/lib/time_range_selection";
  import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
  import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let process: Process;
  export let processAsyncData: ProcessAsyncData;
  export let width: number;
  export let parentCollapsed: boolean;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();

  let processOffsetMs: number;
  let canvas: HTMLCanvasElement | null;
  let ctx: CanvasRenderingContext2D;
  let height: number;
  let intersectionObserver: IntersectionObserver;

  onMount(() => {
    const childStartTime = Date.parse(process.startTime);
    processOffsetMs = childStartTime - rootStartTime;

    if (canvas) {
      const observer = new IntersectionObserver(onIntersection, {
        threshold: [0, 1],
      });
      observer.observe(canvas);
      const context = canvas.getContext("2d");
      if (context) {
        ctx = context;
        draw();
      }
    }
  });

  onDestroy(() => {
    if (intersectionObserver) {
      intersectionObserver.disconnect();
    }
  });

  $: range = $stateStore?.getViewRange();
  $: scopes = $stateStore?.scopes;

  $: height = Math.max(
    spanPixelHeight,
    processAsyncData.maxDepth * spanPixelHeight
  );

  $: if (width || height || scopes || range || $stateStore?.currentSelection) {
    draw();
  }

  async function onIntersection(entries: IntersectionObserverEntry[]) {
    const visible = entries[0].intersectionRatio > 0;
    if (visible) {
      draw();
    }
  }

  async function draw() {
    if (canvas && ctx && !parentCollapsed) {
      await tick();
      drawTasks();
      if ($stateStore.selectionState) {
        DrawSelectedRange(
          canvas,
          ctx,
          $stateStore.selectionState,
          $stateStore.getViewRange()
        );
      }
    }
  }

  function drawTasks() {
    if (!canvas) {
      return;
    }
    const canvasWidth = canvas.clientWidth;
    const [begin, end] = range;
    const invTimeSpan = 1.0 / (end - begin);
    const msToPixelsFactor = invTimeSpan * canvasWidth;

    ctx.font = "15px arial";

    // const testString = "<>_w";
    // const testTextMetrics = ctx.measureText(testString);
    // const characterWidth = testTextMetrics.width / testString.length;
    // const characterHeight = testTextMetrics.actualBoundingBoxAscent;

    const beginTasks = Math.max(
      begin,
      processAsyncData.minMs + processOffsetMs
    );
    const endTasks = Math.min(end, processAsyncData.maxMs + processOffsetMs);
    const beginTasksPixels = (beginTasks - begin) * msToPixelsFactor;
    const endTasksPixels = (endTasks - begin) * msToPixelsFactor;

    ctx.fillStyle = "#F0F0F0";
    ctx.fillRect(0, 0, canvasWidth, height);
    ctx.fillStyle = "#e8e8e8";
    ctx.fillRect(
      beginTasksPixels,
      0,
      endTasksPixels - beginTasksPixels,
      height
    );
  }
</script>

<div class="drag" on:wheel|preventDefault={(e) => wheelDispatch("zoom", e)}>
  <canvas {width} {height} bind:this={canvas} />
</div>

<style>
  div {
    align-self: stretch;
    background-color: #f0f0f0;
    cursor: grab;
  }
</style>
