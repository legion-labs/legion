<script lang="ts">
  import {
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
  } from "@lgn/proto-telemetry/codegen/analytics";

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  export let id: string;

  async function fetchProcess() {
    const { process } = await client.find_process({
      processId: id,
    });

    if (!process) {
      throw new Error(`Process ${id} not found`);
    }

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
  <div>process_id {id}</div>
  {#await fetchProcess() then logEntriesList}
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
  .logentry {
    @apply text-left bg-[#f0f0f0];
  }

  .logentrytime {
    @apply font-bold pr-5;
  }
</style>
