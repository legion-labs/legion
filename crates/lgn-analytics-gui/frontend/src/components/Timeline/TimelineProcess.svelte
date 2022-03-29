<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { formatExecutionTime, formatProcessName } from "@/lib/format";

  import { TimelineTrackCanvasAsyncDrawer } from "./Drawing/TimelineTrackCanvasAsyncDrawer";
  import { TimelineTrackCanvasSyncDrawer } from "./Drawing/TimelineTrackCanvasSyncDrawer";
  import TimelineRow from "./TimelineRow.svelte";
  import TimelineTrack from "./TimelineTrack.svelte";
  import TimelineDebug from "./Tools/TimelineDebug.svelte";
  import {
    asyncTaskName,
    spanPixelHeight as sph,
  } from "./Values/TimelineValues";

  export let process: Process;
  export let stateStore: TimelineStateStore;
  export let rootStartTime: number;

  const wheelDispatcher = createEventDispatcher<{ zoom: WheelEvent }>();
  const processOffsetMs = Date.parse(process.startTime) - rootStartTime;

  let processCollapsed = false;
  let components: TimelineRow[] = [];

  $: threads = Object.values($stateStore.threads).filter(
    (t) => t.streamInfo.processId === process.processId
  );

  $: processAsyncData = $stateStore.processAsyncData[process.processId];

  $: style = processCollapsed
    ? `min-height:${sph}px;max-height:${sph}px;overflow-y:hidden`
    : ``;
</script>

<div on:wheel|preventDefault={(e) => wheelDispatcher("zoom", e)} {style}>
  <div
    class="process mb-1 flex flex-row place-content-between items-center"
    on:click|preventDefault={() => (processCollapsed = !processCollapsed)}
  >
    <span>
      <i class="bi bi-activity" />
      {formatProcessName(process)}
    </span>
    {#if !processCollapsed}
      <div class="flex flex-row gap-1">
        <i
          title="Collapse"
          class="bi-arrows-angle-contract"
          on:click|stopPropagation={() =>
            components.forEach((c) => c.setCollapse(true))}
        />
        <i
          title="Expand"
          class="bi-arrows-angle-expand"
          on:click|stopPropagation={() =>
            components.forEach((c) => c.setCollapse(false))}
        />
        <TimelineDebug store={stateStore} />
      </div>
    {/if}
  </div>
  <div class="thread-container">
    {#if $stateStore}
      {#if processAsyncData}
        <TimelineRow
          bind:this={components[0]}
          {processCollapsed}
          threadName={asyncTaskName}
          maxDepth={processAsyncData.maxDepth}
        >
          <TimelineTrack
            slot="canvas"
            {stateStore}
            dataObject={processAsyncData}
            {processCollapsed}
            maxDepth={processAsyncData.maxDepth}
            on:zoom={(e) => wheelDispatcher("zoom", e.detail)}
            drawerBuilder={() =>
              new TimelineTrackCanvasAsyncDrawer(
                stateStore,
                processOffsetMs,
                processAsyncData
              )}
          />
        </TimelineRow>
      {/if}
      {#each threads as thread, index (thread.streamInfo.streamId)}
        {@const threadName = thread.streamInfo.properties["thread-name"]}
        {@const threadLength = formatExecutionTime(thread.maxMs - thread.minMs)}
        <TimelineRow
          bind:this={components[index + 1]}
          {processCollapsed}
          threadTitle={`${threadName}\n${threadLength}\n${thread.block_ids.length} block(s)`}
          {threadName}
          maxDepth={thread.maxDepth}
        >
          <span class="text text-xs text-slate-300" slot="details"
            >{threadLength} ({thread.block_ids.length} block{thread.block_ids
              .length
              ? "s"
              : ""})
          </span>
          <TimelineTrack
            slot="canvas"
            dataObject={thread}
            {stateStore}
            {processCollapsed}
            maxDepth={thread.maxDepth}
            on:zoom={(e) => wheelDispatcher("zoom", e.detail)}
            drawerBuilder={() =>
              new TimelineTrackCanvasSyncDrawer(
                stateStore,
                processOffsetMs,
                thread
              )}
          />
        </TimelineRow>
      {/each}
    {/if}
  </div>
</div>

<style lang="postcss">
  .process {
    @apply bg-slate-300 text-slate-500 px-1 text-sm cursor-pointer;
    text-align: left;
  }

  .thread-container {
    @apply flex flex-col gap-y-1;
    user-select: none;
  }
</style>
