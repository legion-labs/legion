<script context="module" lang="ts">
  type Thread = {
    streamInfo: Stream;
    spanBlocks: BlockSpansReply[];
    maxDepth: number;
    minMs: number;
    maxMs: number;
  };

  type BeginPan = {
    beginMouseX: number;
    beginMouseY: number;
    viewRange: [number, number];
    beginYOffset: number;
  };

  type LoadingState = {
    requested: number;
    completed: number;
  };
</script>

<script lang="ts">
  import { link } from "svelte-navigator";
  import {
    BlockSpansReply,
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
    ScopeDesc,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { Block } from "@lgn/proto-telemetry/codegen/block";
  import { Process } from "@lgn/proto-telemetry/codegen/process";
  import { Stream } from "@lgn/proto-telemetry/codegen/stream";
  import { onMount } from "svelte";
  import { formatExecutionTime } from "@/lib/format";
  import { zoomHorizontalViewRange } from "@/lib/zoom";
  import TimeRangeDetails from "@/components/TimeRangeDetails.svelte";
  import {
    DrawSelectedRange,
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
    SelectionState,
  } from "@/lib/time_range_selection";

  export let processId: string;

  let timelineStart: number | undefined;
  let timelineEnd: number | undefined;

  let canvas: HTMLCanvasElement | undefined;
  let processList: Process[] = [];
  let currentProcess: Process | undefined;
  let renderingContext: CanvasRenderingContext2D | undefined;
  let minMs = Infinity;
  let maxMs = -Infinity;
  let yOffset = 0;
  let threads: Record<string, Thread> = {};
  let blockList: Block[] = [];
  let scopes: Record<number, ScopeDesc> = {};
  let viewRange: [number, number] | undefined;
  let beginPan: BeginPan | undefined;
  let selectionState: SelectionState = NewSelectionState();
  let currentSelection: [number, number] | undefined;
  let loadingProgression: LoadingState | undefined;

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const startParam = urlParams.get("timelineStart");
    if (startParam) {
      timelineStart = Number.parseFloat(startParam);
    }
    const endParam = urlParams.get("timelineEnd");
    if (endParam) {
      timelineEnd = Number.parseFloat(endParam);
    }

    const canvas = document.getElementById("canvas_timeline");
    if (!canvas || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error("Canvas can't be found or is invalid");
    }

    const context = canvas.getContext("2d");

    if (!context) {
      throw new Error("Couldn't get context for canvas");
    }

    renderingContext = context;
    fetchProcessInfo();
  });

  async function fetchProcessInfo() {
    const { process } = await client.find_process({ processId: processId });

    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }

    processList.push(process);
    await fetchStreams(process);
    currentProcess = process;
    await fetchChildren();
    loadingProgression = { requested: blockList.length, completed: 0 };
    blockList.forEach((block) => fetchBlockSpans(block));
  }

  async function fetchStreams(process: Process) {
    const { streams } = await client.list_process_streams({
      processId: process.processId,
    });

    let promises: Promise<void>[] = [];
    streams.forEach((stream) => {
      if (stream.tags.includes("cpu")) {
        threads[stream.streamId] = {
          streamInfo: stream,
          spanBlocks: [],
          maxDepth: 0,
          minMs: Infinity,
          maxMs: -Infinity,
        };

        promises.push(fetchBlocks(stream.streamId));
      }
    });
    await Promise.all(promises);
  }

  async function fetchChildren() {
    const { processes } = await client.list_process_children({
      processId: processId,
    });

    let promises = processes.map((process) => {
      processList.push(process);
      return fetchStreams(process);
    });
    await Promise.all(promises);
  }

  async function fetchBlocks(streamId: string) {
    const { blocks } = await client.list_stream_blocks({ streamId });
    blockList = blockList.concat(blocks);
  }

  async function fetchBlockSpans(block: Block) {
    const streamId = block.streamId;

    const process = findStreamProcess(streamId);

    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }

    const response = await client.block_spans({
      blockId: block.blockId,
      process,
      stream: threads[streamId].streamInfo,
    });
    scopes = {...scopes,...response.scopes};
    minMs = Math.min(minMs, response.beginMs);
    maxMs = Math.max(maxMs, response.endMs);

    let thread = threads[streamId];
    thread.spanBlocks.push(response);
    thread.maxDepth = Math.max(thread.maxDepth, response.maxDepth);
    thread.minMs = Math.min(thread.minMs, response.beginMs);
    thread.maxMs = Math.max(thread.maxMs, response.endMs);
    if (loadingProgression) {
      loadingProgression.completed += 1;
    }
    updateProgess();
    drawCanvas();
  }

  function findStreamProcess(streamId: string) {
    const stream = threads[streamId].streamInfo;

    return processList.find(
      (process) => process.processId === stream.processId
    );
  }

  function drawCanvas() {
    if (!canvas || !renderingContext) {
      return;
    }

    if (!currentProcess) {
      throw new Error("Current process not set");
    }

    canvas.height =
      window.innerHeight - canvas.getBoundingClientRect().top - 20;

    renderingContext.clearRect(0, 0, canvas.width, canvas.height);

    let threadVerticalOffset = yOffset;

    const parentStartTime = Date.parse(currentProcess?.startTime);

    for (const streamId in threads) {
      const childProcess = findStreamProcess(streamId);

      if (!childProcess) {
        throw new Error("Child process not found");
      }

      const childStartTime = Date.parse(childProcess.startTime);
      const thread = threads[streamId];
      if (thread.spanBlocks.length > 0) {
        drawThread(
          thread,
          threadVerticalOffset,
          childStartTime - parentStartTime
        );
        threadVerticalOffset += (thread.maxDepth + 2) * 20;
      }
    }

    DrawSelectedRange(canvas, renderingContext, selectionState, getViewRange());
  }

  function drawThread(
    thread: Thread,
    threadVerticalOffset: number,
    offsetMs: number
  ) {
    if (!canvas || !renderingContext) {
      return;
    }
    if (threadVerticalOffset > canvas.clientHeight) {
      return;
    }

    const [begin, end] = getViewRange();
    const invTimeSpan = 1.0 / (end - begin);
    const canvasWidth = canvas.clientWidth;
    const msToPixelsFactor = invTimeSpan * canvasWidth;

    renderingContext.font = "15px arial";

    const testString = "<>_w";
    const testTextMetrics = renderingContext.measureText(testString);
    const characterWidth = testTextMetrics.width / testString.length;
    const characterHeight = testTextMetrics.actualBoundingBoxAscent;

    const beginThread = Math.max(begin, thread.minMs + offsetMs);
    const endThread = Math.min(end, thread.maxMs + offsetMs);
    const beginThreadPixels = (beginThread - begin) * msToPixelsFactor;
    const endThreadPixels = (endThread - begin) * msToPixelsFactor;

    renderingContext.fillStyle = "#FCFCFC";
    renderingContext.fillRect(
      0,
      threadVerticalOffset,
      canvasWidth,
      20 * thread.maxDepth
    );
    renderingContext.fillStyle = "#F0F0F0";
    renderingContext.fillRect(
      beginThreadPixels,
      threadVerticalOffset,
      endThreadPixels - beginThreadPixels,
      20 * thread.maxDepth
    );

    thread.spanBlocks.forEach((blockSpans) => {
      if (
        blockSpans.beginMs + offsetMs > end ||
        blockSpans.endMs + offsetMs < begin
      ) {
        return;
      }

      blockSpans.spans.forEach(({ beginMs, endMs, depth, scopeHash }) => {
        if (!renderingContext) {
          throw new Error("Rendering context not available");
        }

        const beginSpan = beginMs + offsetMs;
        const endSpan = endMs + offsetMs;

        if (beginSpan > end || endSpan < begin) {
          return;
        }

        const beginPixels = (beginSpan - begin) * msToPixelsFactor;
        const endPixels = (endSpan - begin) * msToPixelsFactor;
        const callWidth = endPixels - beginPixels;
        if (callWidth < 0.1) {
          return;
        }

        const offsetY = threadVerticalOffset + depth * 20;

        if (depth % 2 === 0) {
          renderingContext.fillStyle = "#fede99";
        } else {
          renderingContext.fillStyle = "#fea446";
        }

        renderingContext.fillRect(beginPixels, offsetY, callWidth, 20);

        const { name } = scopes[scopeHash];

        if (callWidth > characterWidth * 5) {
          const nbChars = Math.floor(callWidth / characterWidth);

          renderingContext.fillStyle = "#000000";

          const extraHeight = 0.5 * (20 - characterHeight);
          const caption = name + " " + formatExecutionTime(endSpan - beginSpan);

          renderingContext.fillText(
            caption.slice(0, nbChars),
            beginPixels + 5,
            offsetY + characterHeight + extraHeight,
            callWidth
          );
        }
      });
    });
  }

  function getViewRange(): [number, number] {
    if (viewRange) {
      return viewRange;
    }

    let start = minMs;
    if (timelineStart) {
      start = timelineStart;
    }
    let end = maxMs;
    if (timelineEnd) {
      end = timelineEnd;
    }
    return [start, end];
  }

  function onPan(event: MouseEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }

    if (!beginPan) {
      beginPan = {
        beginMouseX: event.offsetX,
        beginMouseY: event.offsetY,
        viewRange: getViewRange(),
        beginYOffset: yOffset,
      };
    }

    const factor =
      (beginPan.viewRange[1] - beginPan.viewRange[0]) / canvas.width;
    const offsetMs = factor * (beginPan.beginMouseX - event.offsetX);

    viewRange = [
      beginPan.viewRange[0] + offsetMs,
      beginPan.viewRange[1] + offsetMs,
    ];

    yOffset = beginPan.beginYOffset + event.offsetY - beginPan.beginMouseY;
  }

  function onMouseDown(event: MouseEvent) {
    if (RangeSelectionOnMouseDown(event, selectionState)) {
      currentSelection = selectionState.selectedRange;
      drawCanvas();
    }
  }

  // returns if the view should be updated
  function PanOnMouseMove(event: MouseEvent): boolean {
    if (event.buttons !== 1) {
      beginPan = undefined;
      return false;
    }

    if (!event.shiftKey) {
      onPan(event);
      return true;
    }
    return false;
  }

  function onMouseMove(event: MouseEvent) {
    if (!canvas) {
      return;
    }
    if (
      RangeSelectionOnMouseMove(
        event,
        selectionState,
        canvas,
        getViewRange()
      ) ||
      PanOnMouseMove(event)
    ) {
      currentSelection = selectionState.selectedRange;
      drawCanvas();
    }
  }

  function onZoom(event: WheelEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }
    viewRange = zoomHorizontalViewRange(getViewRange(), canvas.width, event);
    drawCanvas();
  }

  function updateProgess() {
    if (!loadingProgression) {
      return;
    }
    var elem = document.getElementById("loadedProgress");
    if (elem) {
      elem.style.width =
        (loadingProgression.completed * 100) / loadingProgression.requested +
        "%";
    }
    if (loadingProgression.completed == loadingProgression.requested) {
      loadingProgression = undefined;
    }
  }
</script>

<div>
  {#if currentProcess}
    <div>
      <div>{currentProcess.exe} {currentProcess.processId}</div>
      {#if currentProcess.parentProcessId}
        <div class="parent-process">
          <a href={`/timeline/${currentProcess.parentProcessId}`} use:link>
            Parent timeline
          </a>
        </div>
      {/if}
    </div>
  {/if}

  {#if loadingProgression}
    <div id="totalLoadingProgress">
      <div id="loadedProgress">Loading</div>
    </div>
  {/if}

  <canvas
    class="timeline-canvas"
    bind:this={canvas}
    id="canvas_timeline"
    width="1024px"
    on:wheel|preventDefault={onZoom}
    on:mousemove|preventDefault={onMouseMove}
    on:mousedown|preventDefault={onMouseDown}
  />

  <TimeRangeDetails timeRange={currentSelection} {processId} />
</div>

<style lang="postcss">
  .parent-process {
    @apply text-[#ca2f0f] underline;
  }

  .timeline-canvas {
    margin: auto;
    display: inline-block;
  }

  #totalLoadingProgress {
    margin: auto;
    width: 90%;
    background-color: grey;
  }

  #loadedProgress {
    width: 0px;
    background-color: #fea446;
  }
</style>
