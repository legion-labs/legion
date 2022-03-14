<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { findBestLod } from "@/lib/time";
  import { Thread } from "@/lib/Timeline/Thread";
  import { TimelineCaptionItem } from "@/lib/Timeline/TimelineSpanCaptionItem";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
  import { DrawSelectedRange } from "@/lib/time_range_selection";
  import { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";
  import binarySearch from "binary-search";
  import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
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

  onMount(() => {
    const process = stateStore.value.findStreamProcess(
      thread.streamInfo.streamId
    );
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
          track,
          color,
          offsetY,
          processOffsetMs,
          begin,
          end,
          characterWidth,
          characterHeight,
          msToPixelsFactor
        );
      }
    });
  }

  function drawSpanTrack(
    track: SpanTrack,
    color: string,
    offsetY: number,
    processOffsetMs: number,
    beginViewRange: number,
    endViewRange: number,
    characterWidth: number,
    characterHeight: number,
    msToPixelsFactor: number
  ) {
    let firstSpan = binarySearch(
      track.spans,
      beginViewRange - processOffsetMs,
      function (span, needle) {
        if (span.endMs < needle) {
          return -1;
        }
        if (span.beginMs > needle) {
          return 1;
        }
        return 0;
      }
    );
    if (firstSpan < 0) {
      firstSpan = ~firstSpan;
    }

    let lastSpan = binarySearch(
      track.spans,
      endViewRange - processOffsetMs,
      function (span, needle) {
        if (span.beginMs < needle) {
          return -1;
        }
        if (span.endMs > needle) {
          return 1;
        }
        return 0;
      }
    );
    if (lastSpan < 0) {
      lastSpan = ~lastSpan;
    }

    for (let spanIndex = firstSpan; spanIndex < lastSpan; spanIndex += 1) {
      const span = track.spans[spanIndex];
      const beginSpan = span.beginMs + processOffsetMs;
      const endSpan = span.endMs + processOffsetMs;

      const beginPixels = (beginSpan - beginViewRange) * msToPixelsFactor;
      const endPixels = (endSpan - beginViewRange) * msToPixelsFactor;
      const callWidth = endPixels - beginPixels;
      if (callWidth < 0.1) {
        continue;
      }
      ctx.fillStyle = color;
      ctx.globalAlpha = span.alpha / 255;
      ctx.fillRect(beginPixels, offsetY, callWidth, spanPixelHeight);
      ctx.globalAlpha = 1.0;

      if (span.scopeHash != 0) {
        if (callWidth > characterWidth * 5) {
          // const nbChars = Math.floor(callWidth / characterWidth);

          ctx.fillStyle = "#000000";

          const extraHeight = 0.5 * (spanPixelHeight - characterHeight);
          const { name } = scopes[span.scopeHash];
          // const caption = name + " " + formatExecutionTime(endSpan - beginSpan);

          // ctx.fillText(
          //   caption.slice(0, nbChars),
          //   beginPixels + 5,
          //   offsetY + characterHeight + extraHeight,
          //   callWidth
          // );

          writeText(
            callWidth,
            characterWidth,
            Array.from(getCaptions(name, beginSpan, endSpan)),
            beginPixels + 5,
            offsetY + characterHeight + extraHeight
          );
        }
      }
    }
  }

  function* getCaptions(
    caption: string,
    beginSpan: number,
    endSpan: number
  ): Generator<TimelineCaptionItem> {
    const mainColor = "#000000";
    const subColor = "#4d4d4d";
    const defaultFont = "15px arial";
    const split = caption.split("::");
    if (split.length > 1) {
      const first = split.shift();
      yield { value: first ?? "", font: defaultFont, color: subColor };
      let current = null;
      while ((current = split.shift())) {
        yield {
          value: `::${current}`,
          font: defaultFont,
          color: split.length > 0 ? subColor : mainColor,
        };
      }
    } else {
      yield { value: caption, color: mainColor };
    }
    yield {
      value: `  (${formatExecutionTime(endSpan - beginSpan)})`,
      color: subColor,
      font: "12px arial",
      skippable: true,
    };
  }

  function writeText(
    width: number,
    characterWidth: number,
    items: TimelineCaptionItem[],
    x: number,
    y: number
  ) {
    let defaultFillStyle = ctx.fillStyle;
    let defaultFont = ctx.font;
    ctx.save();
    for (const { value, font, color, skippable } of items) {
      ctx.fillStyle = color || defaultFillStyle;
      ctx.font = font || defaultFont;
      const budget = Math.floor(width / characterWidth);
      if (!budget) {
        break;
      }
      if (value.length > budget && skippable) {
        continue;
      }
      const textSlice = value.slice(0, budget);
      ctx.fillText(textSlice, x, y);
      const size = ctx.measureText(textSlice).width;
      x += size;
      width -= size;
    }
    ctx.restore();
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
