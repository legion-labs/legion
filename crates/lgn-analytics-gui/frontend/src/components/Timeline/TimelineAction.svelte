<script lang="ts">
  import { Process } from "@lgn/proto-telemetry/dist/process";
  import { link } from "svelte-navigator";
  export let timeRange: [number, number] | undefined;
  export let processId: string;
  export let process: Process;
</script>

<div class="flex flex-row justify-start text-xs gap-1">
  {#if process.parentProcessId}
    <div class="action bg-orange-700">
      <a href={`/timeline/${process.parentProcessId}`} target="_blank" use:link>
        <i class="bi bi-arrow-up-right-circle" />
        Open Parent Timeline
      </a>
    </div>
  {/if}
  {#if timeRange}
    <div class="action bg-slate-300">
      <a
        href={`/cumulative-call-graph?process=${processId}&begin=${timeRange[0]}&end=${timeRange[1]}`}
        target="_blank"
        use:link
      >
        <i class="bi bi-border-style pr-1" />Cumulative Call Graph
      </a>
    </div>
  {/if}
</div>

<style lang="postcss">
  .action {
    @apply text-gray-100 p-1;
    min-width: 170px;
  }
</style>
