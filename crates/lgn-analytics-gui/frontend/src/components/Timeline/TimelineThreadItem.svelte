<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { Thread } from "@/lib/Timeline/Thread";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
  import { createEventDispatcher } from "svelte";
  import TimelineThread from "./TimelineThread.svelte";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let thread: Thread;
  export let width: number;
  let collapsed = false;
  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  $: threadName = thread.streamInfo.properties["thread-name"];
</script>

{#if thread.block_ids.length > 0}
  <div
    class="flex items-start main"
    style={`${
      collapsed
        ? `max-height:${spanPixelHeight}px`
        : `min-height:${(thread.maxDepth + 1) * spanPixelHeight}px`
    }`}
  >
    <div
      class="thread px-1"
      on:click={() => (collapsed = !collapsed)}
      title={`${threadName}\n${thread.block_ids.length} block(s)`}
    >
      <span class="text">
        <i class={`icon bi bi-arrow-${collapsed ? "up" : "down"}-circle`} />
        <span class="thread-name">{threadName}</span></span
      >
      <span class="text text-xs text-slate-300"
        >{formatExecutionTime(thread.maxMs - thread.minMs)}
      </span>
    </div>
    <TimelineThread
      {thread}
      {stateStore}
      width={width - 0}
      {rootStartTime}
      on:zoom={(e) => wheelDispatch("zoom", e.detail)}
    />
  </div>
{/if}

<style lang="postcss">
  .main {
    overflow-y: hidden;
  }

  .thread-name {
    @apply capitalize;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .text {
    @apply flex;
  }

  .icon {
    @apply pr-1;
  }

  .thread {
    @apply text-sm text-slate-400;
    width: 170px;
    overflow: hidden;
    cursor: pointer;
    background-color: #f0f0f0;
    margin-right: 4px;
    align-self: stretch;
  }
</style>
