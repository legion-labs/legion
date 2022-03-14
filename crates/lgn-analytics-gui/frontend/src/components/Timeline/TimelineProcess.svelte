<script lang="ts">
  import { formatProcessName } from "@/lib/format";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
  import { Process } from "@lgn/proto-telemetry/dist/process";
  import { createEventDispatcher } from "svelte";
  import TimelineThreadItem from "./TimelineThreadItem.svelte";
  import TasksItem from "./TasksItem.svelte";
  export let process: Process;
  export let stateStore: TimelineStateStore;
  export let rootStartTime: number;
  export let width: number;
  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  let collapsed = false;
  let components: TimelineThreadItem[] = [];
  $: threads = Object.values($stateStore.threads).filter(
    (t) => t.streamInfo.processId === process.processId
  );
</script>

<div
  style={collapsed
    ? `min-height:${spanPixelHeight}px;max-height:${spanPixelHeight}px;overflow-y:hidden`
    : ``}
>
  <div
    class="process mb-1 flex flex-row place-content-between items-center"
    on:click|preventDefault={() => (collapsed = !collapsed)}
  >
    <span>
      <i class="bi bi-activity" />
      {formatProcessName(process)}
    </span>
    {#if !collapsed}
      <div>
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
      </div>
    {/if}
  </div>
  <div class="thread-container">
    {#if $stateStore}
      <TasksItem on:zoom={(e) => wheelDispatch("zoom", e.detail)} />
      {#each threads as thread, index (thread.streamInfo.streamId)}
        <TimelineThreadItem
          bind:this={components[index]}
          parentCollapsed={collapsed}
          {thread}
          {stateStore}
          {width}
          {rootStartTime}
          on:zoom={(e) => wheelDispatch("zoom", e.detail)}
        />
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
