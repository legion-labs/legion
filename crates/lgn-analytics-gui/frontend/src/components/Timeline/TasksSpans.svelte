<script lang="ts">
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { DrawSelectedRange } from "@/lib/time_range_selection";
  import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let width: number;
  export let parentCollapsed: boolean;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();

  let canvas: HTMLCanvasElement | null;
  let ctx: CanvasRenderingContext2D;
  let height: number;
  let intersectionObserver: IntersectionObserver;

  onMount(() => {
    console.log("rootStartTime", rootStartTime);
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
  // $: blocks = $stateStore?.blocks;
  $: scopes = $stateStore?.scopes;

  // $: if (thread) {
  //   height = Math.max(spanPixelHeight, thread.maxDepth * spanPixelHeight);
  // }

  $: if (width || height || scopes || range || $stateStore?.currentSelection) {
    draw();
  }

  // $: if (!initialized && thread) {
  //   initialized = true;
  //   draw();
  // }

  async function onIntersection(entries: IntersectionObserverEntry[]) {
    const visible = entries[0].intersectionRatio > 0;
    if (visible) {
      draw();
    }
  }

  async function draw() {
    if (canvas && ctx && !parentCollapsed) {
      await tick();
      drawThread();
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

  function drawThread() {
    if (!canvas) {
      return;
    }
    const canvasWidth = canvas.clientWidth;
    // const [begin, end] = range;
    // const invTimeSpan = 1.0 / (end - begin);
    // const msToPixelsFactor = invTimeSpan * canvasWidth;

    ctx.font = "15px arial";

    // const testString = "<>_w";
    // const testTextMetrics = ctx.measureText(testString);
    // const characterWidth = testTextMetrics.width / testString.length;
    // const characterHeight = testTextMetrics.actualBoundingBoxAscent;

    // const beginThread = Math.max(begin, thread.minMs + processOffsetMs);
    // const endThread = Math.min(end, thread.maxMs + processOffsetMs);
    // const beginThreadPixels = (beginThread - begin) * msToPixelsFactor;
    // const endThreadPixels = (endThread - begin) * msToPixelsFactor;

    ctx.fillStyle = "#F0F0F0";
    ctx.fillRect(0, 0, canvasWidth, height);
    ctx.fillStyle = "#e8e8e8";
    // ctx.fillRect(
    //   beginThreadPixels,
    //   0,
    //   endThreadPixels - beginThreadPixels,
    //   height
    // );
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
