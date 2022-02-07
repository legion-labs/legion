<script lang="ts">
  import {
    GrpcWebImpl,
    PerformanceAnalyticsClientImpl,
    ProcessInstance,
  } from "@lgn/proto-telemetry/dist/analytics";
  import { onMount } from "svelte";
  import { link } from "svelte-navigator";
  import log from "@lgn/web-client/src/lib/log";
  import { grpc } from "@improbable-eng/grpc-web";
  import { getAccessToken } from "@lgn/web-client/src/stores/userInfo";

  async function makeGrpcClient() {
    let metadata = new grpc.Metadata();
    const token = await getAccessToken();
    metadata.set("Authorization", "Bearer " + token);
    const options = { metadata: metadata };
    const client = new PerformanceAnalyticsClientImpl(
      new GrpcWebImpl("http://" + location.hostname + ":9090", options)
    );
    return client;
  }

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
    getRecentProcesses();
  });

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

<div>
  <h1>Legion Performance Analytics</h1>
  <h2>Process List</h2>
  <center>
    <div class="search-div">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        autofocus
        type="text"
        class="search-input"
        placeholder="search exe"
        on:input={onSearchChange}
      />
    </div>
    <table>
      <thead>
        <th>start time</th>
        <th>exe</th>
        <th>username</th>
        <th>computer</th>
        <th>log</th>
        <th>metrics</th>
        <th>timeline</th>
      </thead>
      <tbody>
        {#each processList as { nbCpuBlocks, nbMetricBlocks, nbLogBlocks, processInfo } (processInfo?.processId)}
          <tr>
            <td>{formatLocalTime(processInfo?.startTime)}</td>
            <td>{processInfo?.exe}</td>
            <td>{processInfo?.username}</td>
            <td>{processInfo?.computer}</td>
            <td>
              {#if nbLogBlocks > 0 && processInfo}
                <div>
                  <a href={`/log/${processInfo?.processId}`} use:link> log </a>
                </div>
              {/if}
            </td>
            <td>
              {#if nbMetricBlocks > 0 && processInfo}
                <div>
                  <a href={`/metrics/${processInfo?.processId}`} use:link>
                    metrics
                  </a>
                </div>
              {/if}
            </td>
            <td>
              {#if nbCpuBlocks > 0 && processInfo}
                <div>
                  <a href={`/timeline/${processInfo?.processId}`} use:link>
                    timeline
                  </a>
                </div>
              {/if}
            </td>
          </tr>
        {/each}
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
    width: 95%;
  }

  table tbody {
    @apply overflow-auto;
  }

  table thead {
    @apply bg-[rgb(230,230,230)];
  }

  table th {
    @apply py-1 text-center border border-[rgb(153,153,153)];
    border-style: none;
    text-align: left;
  }

  table tr:nth-child(even) {
    background-color: #f2f2f2;
  }

  table td {
    @apply p-1 text-left border border-[rgb(153,153,153)];
    font-family: monospace;
    border-style: none;
  }

  table td div {
    @apply p-1;
  }

  a {
    @apply text-[#ca2f0f] underline;
  }

  .search-div {
    margin: 10px 0px 10px 0px;
  }

  .search-input {
    border-style: solid;
    border-width: 2px;
    border-radius: 8px;
    text-align: center;
  }
</style>
