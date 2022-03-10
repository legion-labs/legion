<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { Thread } from "@/lib/Timeline/Thread";
  import { getThreadCollapseStyle } from "@/lib/Timeline/TimelineCollapse";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { createEventDispatcher } from "svelte";
  import TimelineThread from "./TimelineThread.svelte";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let thread: Thread;
  export let width: number;
  export let parentCollapsed: boolean;
  let collapsed = false;
  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  $: threadName = thread.streamInfo.properties["thread-name"];
  $: threadLength = formatExecutionTime(thread.maxMs - thread.minMs);

  export function setCollapse(state: boolean) {
    collapsed = state;
  }
</script>

<div
  class="flex items-start main"
  style={getThreadCollapseStyle(thread, collapsed)}
>
  <div
    class="thread px-1"
    on:click={() => (collapsed = !collapsed)}
    title={`${threadName}\n${threadLength}\n${thread.block_ids.length} block(s)`}
  >
    <span class="text">
      <i class={`icon bi bi-${!collapsed ? "eye" : "eye-slash"}-fill`} />
      <span class="thread-name">{threadName}</span></span
    >
    <span class="text text-xs text-slate-300"
      >{threadLength} ({thread.block_ids.length} block{thread.block_ids.length
        ? "s"
        : ""})
    </span>
  </div>
  <TimelineThread
    {stateStore}
    {thread}
    {parentCollapsed}
    {width}
    {rootStartTime}
    on:zoom={(e) => wheelDispatch("zoom", e.detail)}
  />
</div>

<style lang="postcss">
  .main {
    overflow: hidden;
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
    min-width: 170px;
    overflow: hidden;
    cursor: pointer;
    background-color: #f0f0f0;
    margin-right: 4px;
    align-self: stretch;
  }
</style>
