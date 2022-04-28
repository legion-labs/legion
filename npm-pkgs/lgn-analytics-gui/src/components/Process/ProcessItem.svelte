<script lang="ts">
  import { formatDistance } from "date-fns";
  import { link } from "svelte-navigator";

  import type { ProcessInstance } from "@lgn/proto-telemetry/dist/analytics";

  import { makeGrpcClient } from "@/lib/client";
  import { formatProcessName } from "@/lib/format";

  import ProcessComputer from "./ProcessComputer.svelte";
  import ProcessPlatform from "./ProcessPlatform.svelte";
  import User from "./User.svelte";

  export let processInstance: ProcessInstance;
  export let depth: number;
  export let index: number;

  let processes: ProcessInstance[] | undefined = undefined;
  let collapsed = true;

  async function onClick() {
    collapsed = !collapsed;
    if (processes) {
      return;
    }
    const client = makeGrpcClient();
    ({ processes } = await client.list_recent_processes({
      parentProcessId: processInstance.processInfo?.processId,
    }));
  }

  function formatLocalTime(timeStr: string): string {
    const time = new Date(timeStr);
    return time.toLocaleTimeString(navigator.language, {
      timeZoneName: "short",
      hour12: false,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
  }
</script>

{#if processInstance.processInfo}
  <div
    class:bg-surface={index % 2 === 0}
    class:bg-background={index % 2 !== 0}
    class:bg-opacity-80={depth > 0}
    class="text-white flex h-8 rounded-md items-center"
  >
    <div class="w-8 text-center text-white p-x-1 opacity-60">
      {#if processInstance.childCount}
        <i
          class={`bi bi-arrow-${
            collapsed ? "down" : "up"
          }-circle-fill cursor-pointer`}
          on:click={() => onClick()}
        />
      {/if}
    </div>
    <div
      class="w-5/12 xl:w-2/12 truncate hidden md:block"
      style={`padding-left:${depth * 20}px`}
    >
      <User user={processInstance.processInfo.realname ?? ""} />
    </div>
    <div
      class="w-5/12 xl:w-2/12 truncate"
      style={`padding-left:${Math.min(0, depth - 1) * 20}px`}
    >
      {#if depth}
        <i class="bi bi-arrow-return-right pr-1 opacity-40" />
      {/if}
      {formatProcessName(processInstance.processInfo)}
    </div>
    <div class="w-2/12 truncate hidden xl:block">
      <ProcessComputer process={processInstance.processInfo} />
    </div>
    <div class="w-2/12 truncate hidden xl:block">
      <ProcessPlatform process={processInstance.processInfo} />
    </div>
    <!-- <div class="w-2/12 truncate">
      <i class="bi bi-clock-fill text-content-38 mr-1" />
      {formatDistance(new Date(processInstance.lastActivity), new Date(), {
        addSuffix: true,
      })}
    </div> -->
    <div class="w-2/12 pl-4 truncate">
      {formatLocalTime(processInstance.processInfo.startTime)}
      ({formatDistance(
        new Date(processInstance.processInfo.startTime),
        new Date(),
        {
          addSuffix: true,
        }
      )})
    </div>
    <div class="flex ml-auto">
      <div class="w-8">
        {#if processInstance.nbLogBlocks > 0}
          <a href={`/log/${processInstance.processInfo.processId}`} use:link>
            <i
              title="Log ({processInstance.nbLogBlocks})"
              class="bi bi-card-text"
            />
          </a>
        {/if}
      </div>
      <div class="w-8">
        {#if processInstance.nbMetricBlocks > 0}
          <a
            href={`/metrics/${processInstance.processInfo.processId}`}
            use:link
          >
            <i
              title="Metrics ({processInstance.nbMetricBlocks})"
              class="bi bi-graph-up"
            />
          </a>
        {/if}
      </div>
      <div class="w-8">
        {#if processInstance.nbCpuBlocks > 0}
          <a
            href={`/timeline/${processInstance.processInfo.processId}`}
            use:link
          >
            <i
              title="Timeline ({processInstance.nbCpuBlocks})"
              class="bi bi-body-text"
            />
          </a>
        {/if}
      </div>
    </div>
  </div>
  {#if !collapsed && processes}
    {#each processes as processInstance (processInstance.processInfo?.processId)}
      <svelte:self {processInstance} depth={depth + 1} index={index + 1} />
    {/each}
  {/if}
{/if}
