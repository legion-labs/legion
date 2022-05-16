<script lang="ts">
  import { page } from "$app/stores";

  import type { LogEntry } from "@lgn/proto-telemetry/dist/log";
  import { Level } from "@lgn/proto-telemetry/dist/log";
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "@/components/Misc/L10n.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import Table from "@/components/Misc/Table.svelte";
  import { getHttpClientContext, getL10nOrchestratorContext } from "@/contexts";
  import { formatProcessName } from "@/lib/format";

  const MAX_NB_ENTRIES_IN_PAGE = 1000;

  const client = getHttpClientContext();

  const { t } = getL10nOrchestratorContext();

  const processId = $page.params.processId;

  let nbEntries = 0;
  let viewRange: [number, number] = [0, 0];
  let processInfo: Process | null = null;
  let logEntries: LogEntry[] = [];
  let loading = true;

  async function fetchLogEntries(urlParams: URLSearchParams) {
    loading = true;
    const { process } = await client.find_process({
      processId,
    });
    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }
    processInfo = process;

    const { count } = await client.nb_process_log_entries({ processId });
    nbEntries = count;

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

    try {
      const reply = await client.list_process_log_entries({
        process,
        begin,
        end,
      });
      viewRange = [reply.begin, reply.end];
      logEntries = reply.entries;
    } finally {
      loading = false;
    }
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

  // TODO: Use theme colors
  function levelToColorCssVar(level: Level) {
    switch (level) {
      case Level.ERROR: {
        return "--severity-level-error";
      }

      case Level.WARN: {
        return "--severity-level-warn";
      }

      case Level.INFO: {
        return "--severity-level-info";
      }

      case Level.DEBUG: {
        return "--severity-level-debug";
      }

      case Level.TRACE: {
        return "--severity-level-trace";
      }
    }
  }

  $: fetchLogEntries($page.url.searchParams);

  $: processDescription = processInfo
    ? `${processInfo.exe} (${processInfo.processId})`
    : null;

  $: formattedProcessName = processInfo ? formatProcessName(processInfo) : null;
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
          <div title={processDescription}>{formattedProcessName}</div>
        </div>
      </div>
      {#if !loading}
        <div class="pr-4">
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
    {/if}
  </div>
  <div slot="content">
    {#if loading}
      <Loader />
    {:else}
      <div class="log">
        <Table
          columns={{ level: "7%", timeMs: "8%", target: "10%", msg: "75%" }}
          items={logEntries}
        >
          <div
            slot="header"
            class="table-header"
            let:columnName
            title={$t("log-parent-table-column", { columnName })}
          >
            <div class="truncate">
              <L10n id="log-parent-table-column" variables={{ columnName }} />
            </div>
          </div>
          <div slot="cell" class="table-cell" let:columnName let:value>
            {#if columnName === "level"}
              <div
                class="flex flex-row space-x-1 items-center self-start truncate"
                title={$t("global-severity-level", { level: value })}
                style={`color:rgb(var(${levelToColorCssVar(value)}));`}
              >
                <L10n id="global-severity-level" variables={{ level: value }} />
              </div>
            {:else if columnName === "timeMs"}
              <div
                class="truncate flex justify-end w-full"
                title={formatTime(value)}
              >
                <div>
                  {formatTime(value)}
                </div>
              </div>
            {:else if columnName === "target"}
              <div dir="rtl" class="truncate" title={value}>
                {value}
              </div>
            {:else}
              {value}
            {/if}
          </div>
        </Table>
      </div>
    {/if}
  </div>
</Layout>

<style lang="postcss">
  .log {
    @apply flex flex-col background;
  }

  .table-header {
    @apply flex flex-row items-center h-full headline text-base;
  }

  .table-cell {
    /* TODO: Use proper color */
    @apply flex flex-row h-full py-0.5 headline;
  }
</style>
