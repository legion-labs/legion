<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import { getDebugContext, getL10nOrchestratorContext } from "@/contexts";
  import { formatExecutionTime, formatProcessName } from "@/lib/format";

  import L10n from "../Misc/L10n.svelte";
  import { TimelineTrackCanvasAsyncDrawer } from "./Drawing/TimelineTrackCanvasAsyncDrawer";
  import { TimelineTrackCanvasSyncDrawer } from "./Drawing/TimelineTrackCanvasSyncDrawer";
  import type { TimelineStateStore } from "./Stores/TimelineStateStore";
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
  export let index: number;

  const wheelDispatcher = createEventDispatcher<{ zoom: WheelEvent }>();
  const processOffsetMs = Date.parse(process.startTime) - rootStartTime;
  const debug = getDebugContext();
  const { t } = getL10nOrchestratorContext();

  let collapsed = false;
  let components: TimelineRow[] = [];

  $: threads = Object.values($stateStore.threads).filter(
    (t) => t.streamInfo.processId === process.processId
  );

  $: processAsyncData = $stateStore.processAsyncData[process.processId];

  $: validThreadCount = threads.filter((t) => t.block_ids.length > 0).length;

  $: style = collapsed
    ? `min-height:${sph}px;max-height:${sph}px;overflow-y:hidden`
    : ``;

  onMount(() => {
    collapsed = index !== 0;
  });
</script>

<div on:wheel|preventDefault={(e) => wheelDispatcher("zoom", e)} {style}>
  <div
    class="surface headline px-1 text-sm text-left mb-1 flex flex-row place-content-between items-center"
    on:click|preventDefault={() => (collapsed = !collapsed)}
  >
    <div>
      <span>
        <i class="bi bi-activity" />
        {formatProcessName(process)}
      </span>
      {#if collapsed}
        <span class="text-xs placeholder">
          <L10n
            id="timeline-main-collapsed-extra"
            variables={{ validThreadCount }}
          />
        </span>
      {/if}
    </div>
    {#if !collapsed}
      <div class="flex flex-row gap-1">
        <i
          title={$t("timeline-main-collapse")}
          class="bi-arrows-angle-contract"
          on:click|stopPropagation={() =>
            components.forEach((c) => c.setCollapse(true))}
        />
        <i
          title={$t("timeline-main-expand")}
          class="bi-arrows-angle-expand"
          on:click|stopPropagation={() =>
            components.forEach((c) => c.setCollapse(false))}
        />
        {#if $debug}
          <TimelineDebug store={stateStore} />
        {/if}
      </div>
    {/if}
  </div>
  <div class="flex flex-col gap-y-1 select-none">
    {#if $stateStore}
      {#if processAsyncData && Object.keys(processAsyncData.blockStats).length > 0}
        <TimelineRow
          bind:this={components[0]}
          processCollapsed={collapsed}
          threadName={asyncTaskName}
          maxDepth={processAsyncData.maxDepth}
        >
          <TimelineTrack
            slot="canvas"
            {stateStore}
            processCollapsed={collapsed}
            maxDepth={processAsyncData.maxDepth}
            on:zoom={(e) => wheelDispatcher("zoom", e.detail)}
            drawerBuilder={() =>
              new TimelineTrackCanvasAsyncDrawer(
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
          processCollapsed={collapsed}
          threadTitle={$t("timeline-main-thread-description-title", {
            threadName,
            threadBlocks: thread.block_ids.length,
            threadLength,
          })}
          {threadName}
          maxDepth={thread.maxDepth}
        >
          <span class="text-xs placeholder" slot="details">
            <L10n
              id="timeline-main-thread-description"
              variables={{
                threadBlocks: thread.block_ids.length,
                threadLength,
              }}
            />
          </span>
          <TimelineTrack
            slot="canvas"
            {stateStore}
            processCollapsed={collapsed}
            maxDepth={thread.maxDepth}
            on:zoom={(e) => wheelDispatcher("zoom", e.detail)}
            drawerBuilder={() =>
              new TimelineTrackCanvasSyncDrawer(processOffsetMs, thread)}
          />
        </TimelineRow>
      {/each}
    {/if}
  </div>
</div>
