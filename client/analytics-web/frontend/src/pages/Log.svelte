<script lang="ts">
  import { link } from "svelte-navigator";
  import {
    GrpcWebImpl,
    LogEntry,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { Process } from "@lgn/proto-telemetry/codegen/process";
  import { onMount } from "svelte";

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  export let id: string;
  const MAX_NB_ENTRIES_IN_PAGE = 1000;
  let nbEntries = 0;
  let viewRange: [number, number] = [0, 0];
  let processInfo: Process | null = null;
  let logEntries: LogEntry[] = [];

  async function fetchLogEntries() {
    const { process } = await client.find_process({
      processId: id,
    });
    if (!process) {
      throw new Error(`Process ${id} not found`);
    }
    processInfo = process;

    const { count } = await client.nb_process_log_entries({ processId: id });
    nbEntries = count;

    const urlParams = new URLSearchParams(window.location.search);
    let begin = 0;
    const beginParam = urlParams.get("begin");
    if (beginParam) {
      begin = Number.parseFloat(beginParam);
    }

    let end = Math.min(count, MAX_NB_ENTRIES_IN_PAGE);
    const endParam = urlParams.get("end");
    if (endParam) {
      end = Number.parseFloat(endParam);
    }

    const reply = await client.list_process_log_entries({
      process,
      begin,
      end,
    });
    viewRange = [reply.begin, reply.end];
    logEntries = reply.entries;
  }

  onMount(() => {
    fetchLogEntries();
  });

  function formatTime(ms: number) {
    const seconds = ms / 1000;
    const secondsWhole = Math.floor(seconds);
    const secondsStr = String(secondsWhole % 60).padStart(2, "0");
    const secondsFraction = String(Math.round(ms % 1000)).padStart(3, "0");
    const minutes = secondsWhole / 60;
    const minutesWhole = Math.floor(minutes);
    const minutesStr = String(minutesWhole).padStart(2, "0");
    const hours = minutesWhole / 60;
    const hoursWhole = Math.floor(hours);
    const hoursStr = String(hoursWhole).padStart(2, "0");

    return (
      hoursStr + ":" + minutesStr + ":" + secondsStr + "." + secondsFraction
    );
  }
</script>

<div>
  <h1>Legion Performance Analytics</h1>
  <h2>Process Log</h2>
  {#if processInfo}
    <div class="process-details-div">
      <div>process_id: {processInfo.processId}</div>
      <div>exe: {processInfo.exe}</div>

      {#if processInfo.parentProcessId}
        <div class="nav-link">
          <a href={`/log/${processInfo.parentProcessId}`} use:link>
            Parent Process Log
          </a>
        </div>
      {/if}
    </div>
  {/if}
  {#each logEntries as entry, index (index)}
    <div class="logentry">
      <span class="logentrytime">{formatTime(entry.timeMs)}</span>
      <span>{entry.msg}</span>
    </div>
  {/each}

  {#if nbEntries > MAX_NB_ENTRIES_IN_PAGE}
    <div class="page-nav">
      {#if viewRange[0] > 0}
        <span class="nav-link">
          <a
            href={`/log/${id}?begin=0&end=${Math.min(
              MAX_NB_ENTRIES_IN_PAGE,
              nbEntries
            )}`}
            use:link
          >
            First
          </a>
        </span>
        <span class="nav-link">
          <a
            href={`/log/${id}?begin=${Math.max(
              0,
              viewRange[0] - MAX_NB_ENTRIES_IN_PAGE
            )}&end=${viewRange[0]}`}
            use:link
          >
            Previous
          </a>
        </span>
      {/if}
      {#if viewRange[1] < nbEntries}
        <span class="nav-link">
          <a
            href={`/log/${id}?begin=${viewRange[1]}&end=${
              viewRange[1] + MAX_NB_ENTRIES_IN_PAGE
            }`}
            use:link
          >
            Next
          </a>
        </span>
        <span class="nav-link">
          <a
            href={`/log/${id}?begin=${
              nbEntries - MAX_NB_ENTRIES_IN_PAGE
            }&end=${nbEntries}`}
            use:link
          >
            Last
          </a>
        </span>
      {/if}
    </div>
  {/if}
</div>

<style lang="postcss">
  h1 {
    @apply text-2xl;
  }

  h2 {
    @apply text-xl;
  }

  .process-details-div {
    text-align: left;
    margin: 0px 0px 5px 5px;
  }

  .logentry {
    @apply text-left bg-[#f0f0f0];
    margin: 0px 0px 0px 5px;
  }

  .logentrytime {
    @apply font-bold pr-5;
  }

  .nav-link {
    @apply text-[#42b983] underline;
  }

  .page-nav {
    text-align: left;
    margin: 0px 0px 5px 5px;
  }
</style>
