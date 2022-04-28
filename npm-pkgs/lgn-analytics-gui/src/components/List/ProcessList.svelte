<script lang="ts">
  import { onMount } from "svelte";
  import { link } from "svelte-navigator";

  import type {
    PerformanceAnalyticsClientImpl,
    ProcessInstance,
  } from "@lgn/proto-telemetry/dist/analytics";
  import log from "@lgn/web-client/src/lib/log";

  import { makeGrpcClient } from "@/lib/client";
  import { formatProcessName } from "@/lib/format";

  import Loader from "../Misc/Loader.svelte";
  import Computer from "./Computer.svelte";
  import Platform from "./Platform.svelte";
  import ProcessTime from "./ProcessTime.svelte";
  import User from "./User.svelte";

  let client: PerformanceAnalyticsClientImpl | null = null;
  let processList: ProcessInstance[] = [];
  let loading = true;

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
    client = makeGrpcClient();
    await getRecentProcesses();
    loading = false;
  });
</script>

<Loader {loading}>
  <div slot="body" class="text-headline text-sm">
    <div class="text-center pb-6">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        autofocus
        type="text"
        class="search-input h-8 w-96 placeholder-text rounded-sm pl-2 bg-surface"
        placeholder="Search process..."
        on:input={onSearchChange}
      />
    </div>
    <table class="w-full">
      <thead class="select-none">
        <th>User</th>
        <th>Executable</th>
        <th>Computer</th>
        <th>Platform</th>
        <th>Start Time</th>
        <th />
        <th />
        <th />
      </thead>
      <tbody>
        {#each processList as { nbCpuBlocks, nbMetricBlocks, nbLogBlocks, processInfo } (processInfo?.processId)}
          <tr>
            <td><User user={processInfo?.realname ?? ""} /></td>
            <td>
              {#if processInfo}
                <span title={processInfo?.exe}>
                  {formatProcessName(processInfo)}
                </span>
              {/if}
            </td>
            <td><Computer process={processInfo} /></td>
            <td><Platform process={processInfo} /></td>
            <td><ProcessTime process={processInfo} /></td>
            <td>
              {#if nbLogBlocks > 0 && processInfo}
                <div>
                  <a href={`/log/${processInfo?.processId}`} use:link>
                    <i title="Log ({nbLogBlocks})" class="bi bi-card-text" />
                  </a>
                </div>
              {/if}
            </td>
            <td>
              {#if nbMetricBlocks > 0 && processInfo}
                <div>
                  <a href={`/metrics/${processInfo?.processId}`} use:link>
                    <i
                      title="Metrics ({nbMetricBlocks})"
                      class="bi bi-graph-up"
                    />
                  </a>
                </div>
              {/if}
            </td>
            <td>
              {#if nbCpuBlocks > 0 && processInfo}
                <div>
                  <a href={`/timeline/${processInfo?.processId}`} use:link>
                    <i
                      title="Timeline ({nbCpuBlocks}"
                      class="bi bi-body-text"
                    />
                  </a>
                </div>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</Loader>

<style lang="postcss">
  /** TODO: Use divs instead of a table */

  table {
    border-collapse: separate;
    border-spacing: 0 0.2rem;
  }

  table th {
    @apply text-sm capitalize py-1 pl-1 border text-left;

    border-style: none;
  }

  table tr {
    @apply bg-surface rounded-md;
  }

  table tr td:first-child {
    @apply rounded-l-md;
  }

  table tr td:last-child {
    @apply rounded-r-md;
  }

  table td {
    @apply px-2 py-1 border-none;
  }

  table td div {
    @apply p-0;
  }
</style>
