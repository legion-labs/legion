<script lang="ts">
  import { formatDistance } from "date-fns";
  import { getContext } from "svelte";

  import type { ProcessInstance } from "@lgn/proto-telemetry/dist/analytics";
  import HighlightedText from "@lgn/web-client/src/components/HighlightedText.svelte";
  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";

  import { formatProcessName } from "@/lib/format";

  import ProcessComputer from "./ProcessComputer.svelte";
  import ProcessList from "./ProcessList.svelte";
  import ProcessPlatform from "./ProcessPlatform.svelte";
  import User from "./User.svelte";

  const { locale } = getContext(l10nOrchestratorContextKey);
  const client = getContext("http-client");

  export let columns: Record<
    "user" | "process" | "computer" | "platform" | "start-time" | "statistics",
    string
  >;

  export let processInstance: ProcessInstance;

  export let depth: number;

  export let highlightedPattern: string | RegExp | undefined = undefined;

  export let viewMode: "all" | "search";

  /** Prevent folding/unfolding, if this is `true` a process' sub processes can't be displayed */
  export let noFold: boolean;

  let processes: ProcessInstance[] | undefined = undefined;
  let collapsed = true;
  let formattedStateTime: string | undefined;

  async function onClick() {
    collapsed = !collapsed;

    if (processes) {
      return;
    }

    ({ processes } = await $client.list_recent_processes({
      parentProcessId: processInstance.processInfo?.processId,
    }));
  }

  function formatLocalTime(timeStr: string): string {
    const time = new Date(timeStr);

    return time.toLocaleTimeString($locale, {
      timeZoneName: "short",
      hour12: false,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
  }

  $: if ($locale && processInstance.processInfo) {
    const time = formatLocalTime(processInstance.processInfo.startTime);
    const distance = formatDistance(
      new Date(processInstance.processInfo.startTime),
      new Date(),
      {
        addSuffix: true,
      }
    );

    formattedStateTime = `${time} (${distance})`;
  }

  $: formattedProcessName =
    (processInstance.processInfo &&
      formatProcessName(processInstance.processInfo)) ||
    "";
</script>

{#if processInstance.processInfo}
  <div
    class="background text-white flex h-8 items-center border-b border-[#202020] px-1"
  >
    <div
      class="truncate flex flex-row px-0.5"
      style={`width: ${columns.process}`}
      title={processInstance.processInfo.exe}
    >
      <div class="text-center w-6 text-white p-x-1 opacity-60">
        {#if !noFold && processInstance.childCount > 0}
          <i
            class={`bi bi-arrow-${
              collapsed ? "down" : "up"
            }-circle-fill cursor-pointer`}
            on:click={onClick}
          />
        {/if}
      </div>
      {#if depth}
        <div style="padding-left: {(depth - 1) * 0.5}rem">
          <i class="bi bi-arrow-return-right pr-1 opacity-40" />
        </div>
      {/if}
      {#if highlightedPattern}
        <HighlightedText
          pattern={highlightedPattern}
          text={formattedProcessName}
        />
      {:else}
        {formattedProcessName}
      {/if}
    </div>
    <div class="px-0.5" style={`width: ${columns.user}`}>
      <User user={processInstance.processInfo.username ?? ""}>
        <div class="truncate" slot="default" let:user>
          {#if highlightedPattern}
            <HighlightedText pattern={highlightedPattern} text={user} />
          {:else}
            {user}
          {/if}
        </div>
      </User>
    </div>
    <div class="px-0.5" style={`width: ${columns.computer}`}>
      <ProcessComputer process={processInstance.processInfo}>
        <div class="truncate" slot="default" let:computer>
          {#if highlightedPattern}
            <HighlightedText pattern={highlightedPattern} text={computer} />
          {:else}
            {computer}
          {/if}
        </div>
      </ProcessComputer>
    </div>
    <div class="truncate px-0.5" style={`width: ${columns.platform}`}>
      <ProcessPlatform process={processInstance.processInfo} />
    </div>
    <!-- <div class="w-2/12 truncate">
      <i class="bi bi-clock-fill text-content-38 mr-1" />
      {formatDistance(new Date(processInstance.lastActivity), new Date(), {
        addSuffix: true,
      })}
    </div> -->
    <div
      class="truncate px-0.5"
      style={`width: ${columns["start-time"]}`}
      title={formattedStateTime}
    >
      {formattedStateTime}
    </div>
    <div class="px-0.5 flex" style={`width: ${columns.statistics}`}>
      <div class="w-8">
        {#if processInstance.nbLogBlocks > 0}
          <a href={`/log/${processInstance.processInfo.processId}`}>
            <i
              title="Log ({processInstance.nbLogBlocks})"
              class="bi bi-card-text"
            />
          </a>
        {/if}
      </div>
      <div class="w-8">
        {#if processInstance.nbMetricBlocks > 0}
          <a href={`/metrics/${processInstance.processInfo.processId}`}>
            <i
              title="Metrics ({processInstance.nbMetricBlocks})"
              class="bi bi-graph-up"
            />
          </a>
        {/if}
      </div>
      <div class="w-8">
        {#if processInstance.nbCpuBlocks > 0}
          <a href={`/timeline/${processInstance.processInfo.processId}`}>
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
    <ProcessList
      headless
      {highlightedPattern}
      {viewMode}
      depth={depth + 1}
      {processes}
    />
  {/if}
{/if}
