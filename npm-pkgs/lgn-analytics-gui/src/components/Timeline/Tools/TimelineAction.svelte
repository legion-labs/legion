<script lang="ts">
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "@/components/Misc/L10n.svelte";

  export let timeRange: [number, number] | undefined;
  export let processId: string;
  export let process: Process;
</script>

<div class="flex flex-row justify-start text-xs gap-1 w-full">
  {#if process.parentProcessId}
    <div class="action bg-orange-700 hover:bg-orange-800">
      <a href={`/timeline/${process.parentProcessId}`} target="_blank">
        <i class="bi bi-arrow-up-right-circle" />
        <L10n id="timeline-open-cumulative-call-graph" />
      </a>
    </div>
  {/if}
  {#if timeRange}
    <div class="action bg-slate-400 hover:bg-slate-500">
      <a
        href={`/cumulative-call-graph?process=${processId}&begin=${timeRange[0]}&end=${timeRange[1]}`}
        target="_blank"
      >
        <i class="bi bi-border-style pr-1" />
        <L10n id="global-cumulative-call-graph" />
      </a>
    </div>
    <div
      class="action bg-slate-400 cursor-pointer hover:bg-slate-500"
      on:click={() => navigator.clipboard.writeText(window.location.href)}
    >
      <i class="bi bi bi-share-fill pr-1" />
      <L10n id="global-link-copy" />
    </div>
  {/if}
</div>

<style lang="postcss">
  .action {
    @apply text-white py-1 px-2;
  }

  i {
    @apply px-1;
  }
</style>
