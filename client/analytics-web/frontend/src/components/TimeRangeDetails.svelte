<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { link } from "svelte-navigator";

  export let timeRange: [number, number] | undefined;
  export let processId: string;
</script>

<div id="selected-time-range-div">
  {#if timeRange}
    <h3>Selected time range</h3>
    <div>
      <span>duration: </span>
      <span>{formatExecutionTime(timeRange[1] - timeRange[0])}<span /></span>
    </div>
    <div>
      <span>beginning: </span>
      <span>{formatExecutionTime(timeRange[0])}<span /></span>
    </div>
    <div>
      <span>end: </span>
      <span>{formatExecutionTime(timeRange[1])}<span /></span>
    </div>
    <div class="nav-link">
      <a
        href={`/cumulative-call-graph?process=${processId}&begin=${timeRange[0]}&end=${timeRange[1]}`}
        use:link
      >
        Cumulative Call Graph
      </a>
    </div>
    <div class="nav-link">
      <a
        href={`/timeline/${processId}?timelineStart=${timeRange[0]}&timelineEnd=${timeRange[1]}`}
        use:link
      >
        Timeline
      </a>
    </div>
  {/if}
</div>

<style lang="postcss">
  #selected-time-range-div {
    display: inline-block;
    width: 200px;
    text-align: left;
  }

  .nav-link {
    @apply text-[#ca2f0f] underline;
  }
</style>
