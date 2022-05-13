<!-- <script lang="ts" context="module">
  export const load: Load = async ({ fetch, url }) => {
</script> -->
<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  import type { LogEntry } from "@lgn/proto-telemetry/dist/analytics";
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "@/components/Misc/L10n.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import { getHttpClientContext } from "@/contexts";
  import Loader from "@/components/Misc/Loader.svelte";

  const MAX_NB_ENTRIES_IN_PAGE = 1000;

  const client = getHttpClientContext();

  const processId = $page.params.processId;

  let nbEntries = 0;
  let viewRange: [number, number] = [0, 0];
  let processInfo: Process | null = null;
  let logEntries: LogEntry[] = [];
  let loading = true;

  async function fetchLogEntries() {
    const { process } = await client.find_process({
      processId,
    });
    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }
    processInfo = process;

    const { count } = await client.nb_process_log_entries({ processId });
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

  onMount(async () => {
    try {
      await fetchLogEntries();
    } finally {
      loading = false;
    }
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

<Layout>
  <div
    class="flex flex-row justify-between items-center pl-4 w-full"
    slot="sub-header"
  >
    {#if processInfo}
      <div class="flex flex-row space-x-2">
        {#if processInfo.parentProcessId}
          <div class="flex flex-row space-x-2 text">
            <a href={`/log/${processInfo.parentProcessId}`}>
              <L10n id="log-parent-link" />
            </a>
            <div>/</div>
          </div>
        {/if}
        <div>
          <div>{processInfo.processId} ({processInfo.exe})</div>
        </div>
      </div>
      <div>
        {#if nbEntries > MAX_NB_ENTRIES_IN_PAGE}
          <div class="text-primary flex space-x-8 self-center">
            {#if viewRange[0] > 0}
              <div class="flex space-x-4">
                <span class="nav-link">
                  <a
                    href={`/log/${processId}?begin=0&end=${Math.min(
                      MAX_NB_ENTRIES_IN_PAGE,
                      nbEntries
                    )}`}
                  >
                    <L10n id="global-pagination-first" />
                  </a>
                </span>
                <span class="nav-link">
                  <a
                    href={`/log/${processId}?begin=${Math.max(
                      0,
                      viewRange[0] - MAX_NB_ENTRIES_IN_PAGE
                    )}&end=${viewRange[0]}`}
                  >
                    <L10n id="global-pagination-previous" />
                  </a>
                </span>
              </div>
            {/if}
            {#if viewRange[1] < nbEntries}
              <div class="flex space-x-4">
                <span class="nav-link">
                  <a
                    href={`/log/${processId}?begin=${viewRange[1]}&end=${
                      viewRange[1] + MAX_NB_ENTRIES_IN_PAGE
                    }`}
                  >
                    <L10n id="global-pagination-next" />
                  </a>
                </span>
                <span class="nav-link">
                  <a
                    href={`/log/${processId}?begin=${
                      nbEntries - MAX_NB_ENTRIES_IN_PAGE
                    }&end=${nbEntries}`}
                  >
                    <L10n id="global-pagination-last" />
                  </a>
                </span>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>
  <div slot="content">
    {#if loading}
      <Loader />
    {:else}
      <div class="log">
        {#each logEntries as entry, index (index)}
          <div class="flex rounded flex-row gap-x-4">
            <div class="font-bold basis-28 shrink-0">
              {formatTime(entry.timeMs)}
            </div>
            <div>{entry.msg}</div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</Layout>

<style lang="postcss">
  .log {
    @apply flex flex-col space-y-2 pt-4 pb-1 px-2 background;
  }
</style>
