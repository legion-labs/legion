<script lang="ts">
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { createEventDispatcher } from "svelte";
  export let rootStartTime: number;
  export let stateStore: TimelineStateStore;
  export let width: number;
  export let parentCollapsed: boolean;
  import TasksSpans from "./TasksSpans.svelte";
  let collapsed = false;
  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  const threadName = "async tasks";

  export function setCollapse(state: boolean) {
    collapsed = state;
  }
</script>

<div class="flex items-start main">
  <div
    class="thread px-1"
    on:click={() => (collapsed = !collapsed)}
    title={`${threadName}`}
  >
    <span class="text">
      <i class={`icon bi bi-${!collapsed ? "eye" : "eye-slash"}-fill`} />
      <span class="thread-name">{threadName}</span></span
    >
  </div>
  <TasksSpans
    {stateStore}
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
    width: 170px;
    overflow: hidden;
    cursor: pointer;
    background-color: #f0f0f0;
    margin-right: 4px;
    align-self: stretch;
  }
</style>
