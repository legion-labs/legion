<script lang="ts">
  import { link } from "svelte-navigator";
  import {
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";
  import { Process } from "@lgn/proto-telemetry/codegen/process";

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  export let id: string;
  let processInfo: Process | null = null;

  async function fetchLogEntries() {
    const { process } = await client.find_process({
      processId: id,
    });

    if (!process) {
      throw new Error(`Process ${id} not found`);
    }

    processInfo = process;

    const { entries } = await client.list_process_log_entries({
      process,
    });

    return entries;
  }

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
        <div class="parent-process">
          <a href={`/log/${processInfo.parentProcessId}`} use:link>
            Parent Process Log
          </a>
        </div>
      {/if}
      
    </div>
  {/if}
  {#await fetchLogEntries() then logEntriesList}
    {#each logEntriesList as entry, index (index)}
      <div class="logentry">
        <span class="logentrytime">{formatTime(entry.timeMs)}</span>
        <span>{entry.msg}</span>
      </div>
    {/each}
  {:catch error}
    <div>An error occured: {error.message}</div>
  {/await}
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

  .parent-process {
    @apply text-[#42b983] underline;
  }
  
</style>
