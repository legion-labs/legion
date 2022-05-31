<script lang="ts">
  import { getContext } from "svelte";

  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "@/components/Misc/L10n.svelte";

  const notifications = getContext("notifications");

  export let timeRange: [number, number] | undefined;
  export let processId: string;
  export let process: Process;

  function copyLink() {
    navigator.clipboard.writeText(window.location.href);

    notifications.push(Symbol.for("link-copied"), {
      type: "success",
      payload: {
        type: "l10n",
        title: {
          id: "timeline-link-copy-notification-title",
        },
        message: {
          id: "timeline-link-copy-notification-message",
        },
      },
    });
  }
</script>

<div class="flex flex-row justify-start text-xs gap-1 w-full">
  {#if process.parentProcessId}
    <div class="action bg-orange-700 hover:bg-orange-800">
      <a
        class="flex space-x-1 py-1 px-2 h-full w-full"
        href={`/timeline/${process.parentProcessId}`}
        target="_blank"
      >
        <div><i class="bi bi-arrow-up-right-circle" /></div>
        <div><L10n id="timeline-open-cumulative-call-graph" /></div>
      </a>
    </div>
  {/if}
  {#if timeRange}
    <div class="action bg-slate-400 hover:bg-slate-500">
      <a
        class="flex space-x-1 py-1 px-2 h-full w-full"
        href={`/cumulative-call-graph?process=${processId}&begin=${timeRange[0]}&end=${timeRange[1]}`}
        target="_blank"
      >
        <div><i class="bi bi-border-style pr-1" /></div>
        <div><L10n id="global-cumulative-call-graph" /></div>
      </a>
    </div>
    <div
      class="flex space-x-1 py-1 px-2 action bg-slate-400 cursor-pointer hover:bg-slate-500"
      on:click={copyLink}
    >
      <i class="bi bi bi-share-fill pr-1" />
      <L10n id="global-link-copy" />
    </div>
  {/if}
</div>

<style lang="postcss">
  .action {
    @apply text-white;
  }

  i {
    @apply px-1;
  }
</style>
