<script context="module" lang="ts">
  import { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";
  import { BarLoader } from "svelte-loading-spinners";

  type BeginPan = {
    beginMouseX: number;
    beginMouseY: number;
    viewRange: [number, number];
    beginYOffset: number;
  };

  type LoadingState = {
    current: number;
    total: number;
  };
</script>

<script lang="ts">
  import { link } from "svelte-navigator";
  import {
    BlockSpansReply,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/dist/analytics";
  import { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
  import { Process } from "@lgn/proto-telemetry/dist/process";
  import { Stream } from "@lgn/proto-telemetry/dist/stream";
  import { onMount, tick } from "svelte";
  import { formatExecutionTime } from "@/lib/format";
  import { zoomHorizontalViewRange } from "@/lib/zoom";
  import TimeRangeDetails from "@/components/Misc/TimeRangeDetails.svelte";
  import binarySearch from "binary-search";
  import { makeGrpcClient } from "@/lib/client";
  import log from "@lgn/web-client/src/lib/log";
  import {
    DrawSelectedRange,
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
    SelectionState,
  } from "@/lib/time_range_selection";
  import {
    getLodFromPixelSizeMs,
    MergeThresholdForLOD as mergeThresholdForLOD,
  } from "@/lib/lod";
  import { Thread } from "@/lib/Timeline/Thread";
  import {
    LODState,
    ThreadBlock,
    ThreadBlockLOD,
  } from "@/lib/Timeline/ThreadBlock";
  import {
    computePreferredBlockLod,
    findBestLod,
    processMsOffsetToRoot,
    timestampToMs,
  } from "@/lib/time";

  export let processId: string;

  let timelineStart: number | undefined;
  let timelineEnd: number | undefined;

  let canvas: HTMLCanvasElement | undefined;
  let refreshTimer: ReturnType<typeof setTimeout> | null = null;
  let processList: Process[] = [];
  let currentProcess: Process | undefined;
  let renderingContext: CanvasRenderingContext2D | undefined;
  let minMs = Infinity;
  let maxMs = -Infinity;
  let yOffset = 0;
  let threads: Record<string, Thread> = {};
  let blocks: Record<string, ThreadBlock> = {};
  let scopes: Record<number, ScopeDesc> = {
    0: { name: "", filename: "", line: 0, hash: 0 },
  };
  let viewRange: [number, number] | undefined;
  let beginPan: BeginPan | undefined;
  let selectionState: SelectionState = NewSelectionState();
  let currentSelection: [number, number] | undefined;
  let loadingProgression: LoadingState = { current: 0, total: 0 };
  let client: PerformanceAnalyticsClientImpl | null = null;
  let drawTime: number;
  let loading = true;
  let windowInnerWidth: number;

  onMount(async () => {
    client = await makeGrpcClient();
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
    fetchProcessInfo().then(updatePixelSize);
  });

  async function fetchProcessInfo() {
    if (!client) {
      log.error("no client in fetchProcessInfo");
      return;
    }
    const { process } = await client.find_process({ processId: processId });
    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }

    processList.push(process);
    currentProcess = process;
    await fetchStreams(process);
    await fetchChildren(process);
    fetchPreferedLods(loadingProgression);
  }

  function fetchPreferedLods(loadingProgression: LoadingState) {
    let nbMissing = 0;
    let nbLoaded = 0;
    let nbInFlight = 0;
    for (let blockId in blocks) {
      const block = blocks[blockId];
      if (!canvas) {
        return null;
      }
      const preferedLod = computePreferredBlockLod(
        canvas.width,
        getViewRange(),
        block
      );
      if (preferedLod == null) {
        continue;
      }

      if (!block.lods[preferedLod]) {
        //untracked lod until now
        block.lods[preferedLod] = {
          state: LODState.Missing,
          tracks: [],
          lodId: preferedLod,
        };
      }

      if (block.lods[preferedLod].state == LODState.Loaded) {
        nbLoaded += 1;
        continue;
      }

      if (block.lods[preferedLod].state == LODState.Requested) {
        nbInFlight += 1;
        continue;
      }
      nbMissing += 1;
      if (nbInFlight < 8) {
        fetchBlockSpans(blocks[blockId], preferedLod);
        nbInFlight += 1;
      }
    }
    // here would be a good place to load some low-priority block lods
    // like the lod that corresponds to the full time range being visible
    // this would have the added bonus of caching the lod0 of all blocks on the server

    loadingProgression.current = nbLoaded;
    loadingProgression.total = nbLoaded + nbMissing;
  }

  async function fetchStreams(process: Process) {
    if (!client) {
      log.error("no client in fetchStreams");
      return;
    }
    const { streams } = await client.list_process_streams({
      processId: process.processId,
    });

    let promises: Promise<void>[] = [];
    streams.forEach((stream) => {
      if (stream.tags.includes("cpu")) {
        threads[stream.streamId] = {
          streamInfo: stream,
          maxDepth: 0,
          minMs: Infinity,
          maxMs: -Infinity,
          block_ids: [],
        };

        promises.push(fetchBlocks(process, stream));
      }
    });
    await Promise.all(promises);
  }

  async function fetchChildren(process: Process) {
    if (!client) {
      log.error("no client in fetchChildren");
      return;
    }
    const { processes } = await client.list_process_children({
      processId: process.processId,
    });

    // commented-out - children will be collapsed by default
    // we should really fetch all the descendents server-side to accomplish this in fewer queries
    // for (let i = 0; i < processes.length; ++i) {
    //   await fetchChildren(processes[i]);
    // }

    let promises = processes.map((process) => {
      processList.push(process);
      return fetchStreams(process);
    });
    await Promise.all(promises);
  }

  async function fetchBlocks(process: Process, stream: Stream) {
    if (!client) {
      log.error("no client in fetchBlocks");
      return;
    }
    const processOffset = processMsOffsetToRoot(currentProcess, process);
    const response = await client.list_stream_blocks({
      streamId: stream.streamId,
    });
    for (let i = 0; i < response.blocks.length; i += 1) {
      let block = response.blocks[i];
      let beginMs = processOffset + timestampToMs(process, block.beginTicks);
      let endMs = processOffset + timestampToMs(process, block.endTicks);
      minMs = Math.min(minMs, beginMs);
      maxMs = Math.max(maxMs, endMs);
      nbEventsRepresented += block.nbObjects;
      const asyncStatsReply = await client.fetch_block_async_stats({
        process,
        stream,
        blockId: block.blockId,
      });
      // console.log(asyncStatsReply);
      blocks[block.blockId] = {
        blockDefinition: block,
        beginMs: beginMs,
        endMs: endMs,
        lods: [],
        asyncStats: asyncStatsReply,
      };
    }
  }

  function onLodReceived(response: BlockSpansReply) {
    loading = false;
    const blockId = response.blockId;
    if (!response.lod) {
      throw new Error(`Error fetching spans for block ${blockId}`);
    }
    scopes = { ...scopes, ...response.scopes };

    const block = blocks[response.blockId];
    let thread = threads[block.blockDefinition.streamId];
    thread.maxDepth = Math.max(thread.maxDepth, response.lod.tracks.length);
    thread.minMs = Math.min(thread.minMs, response.beginMs);
    thread.maxMs = Math.max(thread.maxMs, response.endMs);
    thread.block_ids.push(blockId);
    block.lods[response.lod.lodId].state = LODState.Loaded;
    block.lods[response.lod.lodId].tracks = response.lod.tracks;
    updateProgess();
    invalidateCanvas();
    fetchPreferedLods(loadingProgression);
  }

  function fetchBlockSpans(block: ThreadBlock, lodToFetch: number) {
    if (!client) {
      log.error("no client in fetchBlockSpans");
      return;
    }
    const streamId = block.blockDefinition.streamId;
    const process = findStreamProcess(streamId);
    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }
    block.lods[lodToFetch].state = LODState.Requested;
    const blockId = block.blockDefinition.blockId;
    const fut = client.block_spans({
      blockId: blockId,
      process,
      stream: threads[streamId].streamInfo,
      lodId: lodToFetch,
    });
    fut.then(onLodReceived, (e) => {
      console.log("Error fetching block spans", e);
    });
  }

  function findStreamProcess(streamId: string) {
    const stream = threads[streamId].streamInfo;

    return processList.find(
      (process) => process.processId === stream.processId
    );
  }

  function invalidateCanvas() {
    if (refreshTimer) {
      clearTimeout(refreshTimer);
    }
    refreshTimer = setTimeout(drawCanvas, 10);
  }

  async function drawCanvas() {
    const startTime = performance.now();

    if (!canvas || !renderingContext) {
      return;
    }

    if (!currentProcess) {
      throw new Error("Current process not set");
    }

    await tick();
    canvas.width = Math.max(400, Math.round(windowInnerWidth * 0.95));
    canvas.height = Math.max(400, window.innerHeight * 0.75);

    renderingContext.clearRect(0, 0, canvas.width, canvas.height);
    let threadVerticalOffset = yOffset;

    const rootStartTime = Date.parse(currentProcess?.startTime);

    for (const streamId in threads) {
      const childProcess = findStreamProcess(streamId);

      if (!childProcess) {
        throw new Error("Child process not found");
      }

      const childStartTime = Date.parse(childProcess.startTime);
      const thread = threads[streamId];
      if (thread.block_ids.length > 0) {
        const threadHeight = (thread.maxDepth + 2) * 20;
        if (
          threadVerticalOffset < canvas.height &&
          threadVerticalOffset + threadHeight >= 0
        ) {
          drawThread(
            thread,
            threadVerticalOffset,
            childStartTime - rootStartTime
          );
        }
        threadVerticalOffset += threadHeight;
      }
    }

    DrawSelectedRange(canvas, renderingContext, selectionState, getViewRange());

    drawTime = Math.floor(performance.now() - startTime);
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
    if (!renderingContext) {
      throw new Error("Rendering context not available");
    }

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
      renderingContext.fillStyle = color;
      renderingContext.globalAlpha = span.alpha / 255;
      renderingContext.fillRect(beginPixels, offsetY, callWidth, 20);
      renderingContext.globalAlpha = 1.0;

      if (span.scopeHash != 0) {
        const { name } = scopes[span.scopeHash];
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
      }
    }
  }

  function drawThread(
    thread: Thread,
    threadVerticalOffset: number,
    processOffsetMs: number
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

    const beginThread = Math.max(begin, thread.minMs + processOffsetMs);
    const endThread = Math.min(end, thread.maxMs + processOffsetMs);
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

    thread.block_ids.forEach((block_id) => {
      let block = blocks[block_id];
      let lodToRender = !canvas
        ? null
        : findBestLod(canvas.width, getViewRange(), block);

      if (block.beginMs > end || block.endMs < begin) {
        return;
      }

      if (!lodToRender) {
        return;
      }

      if (!renderingContext) {
        throw new Error("Rendering context not available");
      }

      for (
        let trackIndex = 0;
        trackIndex < lodToRender.tracks.length;
        trackIndex += 1
      ) {
        let track = lodToRender.tracks[trackIndex];
        const offsetY = threadVerticalOffset + trackIndex * 20;
        let color = "";
        if (trackIndex % 2 === 0) {
          color = "#fede99";
        } else {
          color = "#fea446";
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
      invalidateCanvas();
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
      if (currentSelection != selectionState.selectedRange) {
        currentSelection = selectionState.selectedRange;
      }
      fetchPreferedLods(loadingProgression);
      invalidateCanvas();
    }
  }

  function onZoom(event: WheelEvent) {
    if (!canvas) {
      throw new Error("Canvas can't be found");
    }
    viewRange = zoomHorizontalViewRange(getViewRange(), canvas.width, event);
    fetchPreferedLods(loadingProgression);
    invalidateCanvas();
    updatePixelSize();
  }

  function updatePixelSize() {
    if (!canvas) {
      return;
    }
    let vr = getViewRange();
    pixelSize = (vr[1] - vr[0]) / canvas.width;
  }

  function updateProgess() {
    if (!loadingProgression) {
      return;
    }
    var elem = document.getElementById("loadedProgress");
    if (elem) {
      elem.style.width =
        (loadingProgression.current * 100) / loadingProgression.total + "%";
    }
  }

  //debug variables (displayed in debug div)
  let pixelSize = 0;
  let LOD = 0;
  let mergeThreshold = 0;
  let nbEventsRepresented = 0;
  $: {
    LOD = getLodFromPixelSizeMs(pixelSize);
  }

  $: {
    mergeThreshold = mergeThresholdForLOD(LOD);
  }

  $: {
    if (windowInnerWidth) {
      drawCanvas();
    }
  }

  $: display = `display:${loading ? "none" : "block"}`;
</script>

<svelte:window bind:innerWidth={windowInnerWidth} />

<div>
  {#if currentProcess}
    <div style={display}>
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

  {#if loading}
    <div class="flex items-center justify-center loader">
      <BarLoader />
    </div>
  {/if}

  <canvas
    style={display}
    class="timeline-canvas shadow-sm"
    bind:this={canvas}
    id="canvas_timeline"
    width={windowInnerWidth}
    on:wheel|preventDefault={onZoom}
    on:mousemove|preventDefault={onMouseMove}
    on:mousedown|preventDefault={onMouseDown}
  />

  <div style={display}>
    <TimeRangeDetails timeRange={currentSelection} {processId} />
    <div id="debugdiv">
      <div>Drawtime: {drawTime} ms</div>
      <div>
        <span>Pixel Size</span>
        <span>{formatExecutionTime(pixelSize)}</span>
      </div>
      <div>
        <span>LOD</span>
        <span>{LOD}</span>
      </div>
      <div>
        <span>Merge Threshold</span>
        <span>{formatExecutionTime(mergeThreshold)}</span>
      </div>
      <div>
        <span>Nb Events Represented</span>
        <span>{nbEventsRepresented}</span>
      </div>
    </div>
    {#if loadingProgression}
      <div id="totalLoadingProgress">
        <div id="loadedProgress" />
      </div>
    {/if}
  </div>
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
    height: 10px;
    background-color: #fea446;
  }

  #debugdiv {
    margin: 20px 0px 0px 0px;
    text-align: left;
  }

  .loader {
    height: 90vh;
  }
</style>
