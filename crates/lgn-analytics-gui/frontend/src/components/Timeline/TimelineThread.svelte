<script lang="ts">
  import { findBestLod } from "@/lib/time";
  import { Thread } from "@/lib/Timeline/Thread";
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
  import { DrawSelectedRange } from "@/lib/time_range_selection";
  import { drawSpanTrack } from "@/lib/Timeline/SpanRender";
  import { debounced } from "@lgn/web-client/src/lib/store";
  import {
    createEventDispatcher,
    getContext,
    onDestroy,
    onMount,
    tick,
  } from "svelte";
  import { TimelineContext } from "@/lib/Timeline/TimelineContext";
  import { Unsubscriber } from "svelte/store";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let thread: Thread;
  export let width: number;
  export let parentCollapsed: boolean;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();

  let processOffsetMs: number;
  let canvas: HTMLCanvasElement | null;
  let ctx: CanvasRenderingContext2D;
  let height: number;
  let initialized = false;
  let intersectionObserver: IntersectionObserver;
  const searchStore = debounced(TimelineContext.search, 100);
  let searchSubscription: Unsubscriber;

  onMount(() => {
    searchSubscription = searchStore.subscribe((_) => {
      draw();
    });
    const process = $stateStore.findStreamProcess(thread.streamInfo.streamId);
    if (!process) {
      return;
    }
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
    if (searchSubscription) {
      searchSubscription();
    }
  });

  $: range = $stateStore?.getViewRange();
  $: blocks = $stateStore?.blocks;
  $: scopes = $stateStore?.scopes;

  $: if (thread) {
    height = Math.max(spanPixelHeight, thread.maxDepth * spanPixelHeight);
  }

  $: if (width || height || scopes || range || $stateStore?.currentSelection) {
    draw();
  }

  $: if (!initialized && thread) {
    initialized = true;
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
    const [begin, end] = range;
    const invTimeSpan = 1.0 / (end - begin);
    const canvasWidth = canvas.clientWidth;
    const msToPixelsFactor = invTimeSpan * canvasWidth;

    ctx.font = "15px arial";

    const testString = "<>_w";
    const testTextMetrics = ctx.measureText(testString);
    const characterWidth = testTextMetrics.width / testString.length;
    const characterHeight = testTextMetrics.actualBoundingBoxAscent;

    const beginThread = Math.max(begin, thread.minMs + processOffsetMs);
    const endThread = Math.min(end, thread.maxMs + processOffsetMs);
    const beginThreadPixels = (beginThread - begin) * msToPixelsFactor;
    const endThreadPixels = (endThread - begin) * msToPixelsFactor;

    ctx.fillStyle = "#F0F0F0";
    ctx.fillRect(0, 0, canvasWidth, height);
    ctx.fillStyle = "#e8e8e8";
    ctx.fillRect(
      beginThreadPixels,
      0,
      endThreadPixels - beginThreadPixels,
      height
    );

    thread.block_ids.forEach((block_id) => {
      let block = blocks[block_id];
      let lodToRender = !canvas
        ? null
        : findBestLod(canvas.width, range, block);

      if (block.beginMs > end || block.endMs < begin || !lodToRender) {
        return;
      }

      for (
        let trackIndex = 0;
        trackIndex < lodToRender.tracks.length;
        trackIndex += 1
      ) {
        let track = lodToRender.tracks[trackIndex];
        const offsetY = trackIndex * spanPixelHeight;
        let color = "";
        if (trackIndex % 2 === 0) {
          color = "#fea446";
        } else {
          color = "#fede99";
        }

        drawSpanTrack(
          ctx,
          scopes,
          track,
          color,
          offsetY,
          processOffsetMs,
          begin,
          end,
          characterWidth,
          characterHeight,
          msToPixelsFactor,
          $searchStore
        );
      }
    });
  }
</script>

<div
  class="timeline-item"
  style={`width:${width}px`}
  on:wheel|preventDefault={(e) => wheelDispatch("zoom", e)}
>
  <canvas {width} {height} bind:this={canvas} />
</div>

<style>
  div {
    align-self: stretch;
    background-color: #f0f0f0;
    cursor: grab;
  }
</style>
