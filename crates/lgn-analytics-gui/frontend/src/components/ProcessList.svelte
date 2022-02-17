<script lang="ts">
  import {
    PerformanceAnalyticsClientImpl,
    ProcessInstance,
  } from "@lgn/proto-telemetry/dist/analytics";
  import { onMount } from "svelte";
  import log from "@lgn/web-client/src/lib/log";
  import { makeGrpcClient } from "@/lib/client";
  import User from "./User.svelte";
  import Platform from "./Platform.svelte";
  import ProcessTime from "./ProcessTime.svelte";

  let client: PerformanceAnalyticsClientImpl | null = null;
  let processList: ProcessInstance[] = [];

  async function getRecentProcesses() {
    if (!client) {
      log.error("grpc client not initialized");
      processList = [];
      return;
    }
    const response = await client.list_recent_processes({});
    processList = response.processes;
  }

  async function onSearchChange(
    evt: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    if (!client) {
      log.error("grpc client not initialized");
      processList = [];
      return;
    }
    const searchString = evt.currentTarget.value;
    const response = await client.search_processes({ search: searchString });
    processList = response.processes;
  }

  onMount(async () => {
    client = await makeGrpcClient();
    await getRecentProcesses();
  });
</script>

<div>
  <center>
    <div class="search-div">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        autofocus
        type="text"
        class="search-input"
        placeholder="Search process..."
        on:input={onSearchChange}
      />
    </div>
    <table>
      <thead>
        <th>User</th>
        <th>Executable</th>
        <th />
        <th />
        <th />
        <th>Target</th>
        <th>Platform</th>
        <th>Start Time</th>
      </thead>
      <tbody>
        {#each processList as { nbCpuBlocks, nbMetricBlocks, nbLogBlocks, processInfo } (processInfo?.processId)}
          <tr>
            <td><User user={processInfo?.realname ?? ""} /></td>
            <td
              ><span title={processInfo?.exe}>
                {processInfo?.exe.split("/").pop()?.split("\\").pop()}
              </span>
            </td>
            <td>
              {#if nbLogBlocks > 0 && processInfo}
                <div>
                  <a href={`/log/${processInfo?.processId}`}
                    ><i
                      title="Log ({nbLogBlocks})"
                      class="bi bi-card-text"
                    /></a
                  >
                </div>
              {/if}
            </td>
            <td>
              {#if nbMetricBlocks > 0 && processInfo}
                <div>
                  <a href={`/metrics/${processInfo?.processId}`}
                    ><i
                      title="Metrics ({nbMetricBlocks})"
                      class="bi bi-graph-up"
                    /></a
                  >
                </div>
              {/if}
            </td>
            <td>
              {#if nbCpuBlocks > 0 && processInfo}
                <div>
                  <a href={`/timeline/${processInfo?.processId}`}
                    ><i
                      title="Timeline ({nbCpuBlocks})"
                      class="bi bi-body-text"
                    /></a
                  >
                </div>
              {/if}
            </td>
            <td>N/A</td>
            <td><Platform process={processInfo} /></td>
            <td><ProcessTime process={processInfo} /></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </center>
</div>

<style lang="postcss">
  table {
    @apply border-collapse;
    width: 100%;
  }

  table tbody {
    @apply overflow-auto;
  }

  table thead {
    @apply bg-[rgb(230,230,230)];
  }

  table th {
    @apply py-1 pl-1 text-center border border-[rgb(153,153,153)];
    border-style: none;
    text-align: left;
  }

  table tr:nth-child(even) {
    background-color: #f2f2f2;
  }

  table td {
    @apply p-1 text-left border border-[rgb(153,153,153)];
    border-style: none;
    @apply text-sm;
  }

  table th {
    text-transform: capitalize;
    @apply text-sm;
  }

  table td div {
    @apply p-0;
  }

  a {
    @apply text-[#000000] underline;
  }

  .search-div {
    margin: 10px 0px 10px 0px;
  }

  .search-input {
    border-style: solid;
    border-width: 2px;
    padding-left: 4px;
    border-color: rgb(175, 175, 175);
    min-width: 400px;
  }
</style>
