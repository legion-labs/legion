<script lang="ts">
  import {
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
  } from "@/proto/analytics";
  import { link } from "svelte-navigator";

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );

  async function getProcessesList() {
    const response = await client.list_recent_processes({});

    return response.processes;
  }
</script>

<div>
  <h1>Legion Performance Analytics</h1>
  <h2>Process List</h2>
  <center>
    <table>
      <thead>
        <th>Start Time</th>
        <th>id</th>
        <th>exe</th>
        <th>parent id</th>
        <th>timeline</th>
      </thead>
      <tbody>
        {#await getProcessesList() then processList}
          {#each processList as { nbCpuBlocks, nbLogBlocks, processInfo } (processInfo?.processId)}
            <tr>
              <td>{processInfo?.startTime}</td>
              <td>{processInfo?.processId}</td>
              <td>{processInfo?.exe}</td>
              <td>{processInfo?.parentProcessId}</td>
              <td>
                {#if nbCpuBlocks > 0 && processInfo}
                  <div>
                    <a href={`/timeline/${processInfo?.processId}`} use:link>
                      timeline
                    </a>
                  </div>
                {/if}
                {#if nbLogBlocks > 0 && processInfo}
                  <div>
                    <a href={`/log/${processInfo?.processId}`} use:link>
                      log
                    </a>
                  </div>
                {/if}
              </td>
            </tr>
          {/each}
        {:catch}
          <tr><td>Error in list_recent_processes</td></tr>
        {/await}
      </tbody>
    </table>
  </center>
</div>

<style lang="postcss">
  h1 {
    @apply text-2xl;
  }

  h2 {
    @apply text-xl;
  }

  table {
    @apply border-collapse;
  }

  table tbody {
    @apply overflow-auto;
  }

  table thead {
    @apply bg-[rgb(230,230,230)];
  }

  table th {
    @apply py-1 text-center border border-[rgb(153,153,153)];
  }

  table td {
    @apply p-1 text-left border border-[rgb(153,153,153)];
    font-family: monospace;
  }

  table td div {
    @apply p-1;
  }

  a {
    @apply text-[#42b983] underline;
  }
</style>
