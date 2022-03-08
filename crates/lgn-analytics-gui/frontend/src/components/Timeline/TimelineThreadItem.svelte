<script lang="ts">
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
  $: threadName = thread.streamInfo.streamId;
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
      title={threadName}
    >
      <span class="text">
        <i
          class={`icon bi bi-arrow-${collapsed ? "up" : "down"}-circle`}
        />{threadName}</span
      >
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

  .text {
    float: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .icon {
    @apply pr-1;
    float: left;
  }

  .thread {
    @apply text-sm text-slate-400;
    max-width: 150px;
    overflow: hidden;
    cursor: pointer;
    background-color: #f0f0f0;
    margin-right: 4px;
    align-self: stretch;
  }
</style>
