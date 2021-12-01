<script context="module" lang="ts">
  type Thread = {
    streamInfo: Stream;
    spanBlocks: BlockSpansReply[];
  };

  type BeginPan = {
    beginMouseX: number;
    beginMouseY: number;
    viewRange: [number, number];
    beginYOffset: number;
  };

  type BeginDetect = {
    beginMouseX: number;
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

  export let id: string;

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
  let selectedRange: [number, number] | undefined;
  let viewRange: [number, number] | undefined;
  let beginPan: BeginPan | undefined;
  let beginSelect: BeginDetect | undefined;

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  onMount(() => {
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
    try {
      const { process } = await client.find_process({ processId: id });

      if (!process) {
        throw new Error(`Process ${id} not found`);
      }

      processList.push(process);
      fetchStreams(process);
      currentProcess = process;
      fetchChildren();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function fetchStreams(process: Process) {
    try {
      const { streams } = await client.list_process_streams({
        processId: process.processId,
      });

      streams.forEach((stream) => {
        if (stream.tags.includes("cpu")) {
          threads[stream.streamId] = {
            streamInfo: stream,
            spanBlocks: [],
          };

          fetchBlocks(stream.streamId);
        }
      });
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function fetchChildren() {
    try {
      const { processes } = await client.list_process_children({
        processId: id,
      });

      processes.forEach((process) => {
        processList.push(process);

        fetchStreams(process);
      });
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function fetchBlocks(streamId: string) {
    try {
      const { blocks } = await client.list_stream_blocks({ streamId });

      blockList = blockList.concat(blocks);

      blocks.forEach(fetchBlockSpans);
    } catch (error) {
      console.error(error);
      throw error;
    }
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

    response.scopes.forEach((scopeDesc) => {
      scopes[scopeDesc.hash] = scopeDesc;
    });

    minMs = Math.min(minMs, response.beginMs);
    maxMs = Math.max(maxMs, response.endMs);

    threads = {
      ...threads,
      [streamId]: {
        ...threads[streamId],
        spanBlocks: [...threads[streamId].spanBlocks, response],
      },
    };

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

      const maxDepth = drawThread(
        threads[streamId],
        threadVerticalOffset,
        childStartTime - parentStartTime
      );

      if (maxDepth) {
        threadVerticalOffset += (maxDepth + 2) * 20;
      }
    }

    drawSelectedRange();
  }

  function drawSelectedRange() {
    if (!canvas || !renderingContext) {
      return;
    }

    if (!selectedRange) {
      return;
    }

    const [begin, end] = getViewRange();
    const invTimeSpan = 1.0 / (end - begin);
    const canvasWidth = canvas.clientWidth;
    const canvasHeight = canvas.clientHeight;
    const msToPixelsFactor = invTimeSpan * canvasWidth;
    const [beginSelection, endSelection] = selectedRange;
    const beginPixels = (beginSelection - begin) * msToPixelsFactor;
    const endPixels = (endSelection - begin) * msToPixelsFactor;

    renderingContext.fillStyle = "rgba(64, 64, 200, 0.2)";
    renderingContext.fillRect(
      beginPixels,
      0,
      endPixels - beginPixels,
      canvasHeight
    );
  }

  function drawThread(
    thread: Thread,
    threadVerticalOffset: number,
    offsetMs: number
  ) {
    if (!canvas || !renderingContext) {
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

    let maxDepth = 0;

    thread.spanBlocks.forEach((blockSpans) => {
      maxDepth = Math.max(maxDepth, blockSpans.maxDepth);

      blockSpans.spans.forEach(({ beginMs, endMs, depth, scopeHash }) => {
        if (!renderingContext) {
          throw new Error("Rendering context not available");
        }

        const beginSpan = beginMs + offsetMs;
        const endSpan = endMs + offsetMs;
        const beginPixels = (beginSpan - begin) * msToPixelsFactor;
        const endPixels = (endSpan - begin) * msToPixelsFactor;
        const callWidth = endPixels - beginPixels;

        const offsetY = threadVerticalOffset + depth * 20;

        if (depth % 2 === 0) {
          renderingContext.fillStyle = "#7DF9FF";
        } else {
          renderingContext.fillStyle = "#A0A0CC";
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

    return maxDepth;
  }

  function getViewRange(): [number, number] {
    if (viewRange) {
      return viewRange;
    }

    return [minMs, maxMs];
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
    drawCanvas();
  }

  function onSelectRange(event: MouseEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }

    if (!beginSelect) {
      beginSelect = {
        beginMouseX: event.offsetX,
      };
    }

    const viewRange = getViewRange();
    const factor = (viewRange[1] - viewRange[0]) / canvas.width;
    const beginTime = viewRange[0] + factor * beginSelect.beginMouseX;
    const endTime = viewRange[0] + factor * event.offsetX;

    selectedRange = [beginTime, endTime];

    drawCanvas();
  }

  function onMouseDown(event: MouseEvent) {
    if (event.shiftKey) {
      beginSelect = undefined;
      selectedRange = undefined;
      drawCanvas();
    }
  }

  function onMouseMove(event: MouseEvent) {
    if (event.buttons !== 1) {
      beginPan = undefined;
      beginSelect = undefined;

      return;
    }

    if (event.shiftKey) {
      onSelectRange(event);
    } else {
      onPan(event);
    }
  }

  function onZoom(event: WheelEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }

    const speed = 0.75;
    const factor = event.deltaY > 0 ? 1.0 / speed : speed;
    const oldRange = getViewRange();
    const length = oldRange[1] - oldRange[0];
    const newLength = length * factor;
    const pctCursor = event.offsetX / canvas.width;
    const pivot = oldRange[0] + length * pctCursor;

    viewRange = [
      pivot - newLength * pctCursor,
      pivot + newLength * (1 - pctCursor),
    ];

    drawCanvas();
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
      {#if selectedRange}
        <button class="call-graph-button">
          <a href={`/cumulative-call-graph?process=${currentProcess.processId}&begin=${selectedRange[0]}&end=${selectedRange[1]}`} use:link>
            Cumulative Call Graph
          </a>
        </button>
      {/if}
    </div>
  {/if}
  <canvas class="timeline-canvas"
          bind:this={canvas}
          id="canvas_timeline"
          width="1024px"
          on:wheel|preventDefault={onZoom}
          on:mousemove={onMouseMove}
          on:mousedown={onMouseDown}
          />
</div>

<style lang="postcss">
  .parent-process {
    @apply text-[#42b983] underline;
  }

  .timeline-canvas {
    margin: auto;
  }

  .call-graph-button {
    background-color: rgba(64, 64, 200, 0.2);
    border: 1px solid;
    transition-duration: 0.4s;
    border-radius: 4px;
  }

  .call-graph-button:hover {
    background-color: rgba(64, 64, 200, 1.0);
    color: white;
    border: 1px solid;
  }  
</style>
