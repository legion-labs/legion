<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  import type { LogEntry } from "@lgn/proto-telemetry/dist/log";
  import { Level } from "@lgn/proto-telemetry/dist/log";
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "@/components/Misc/L10n.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import Pagination from "@/components/Misc/Pagination.svelte";
  import Table from "@/components/Misc/Table.svelte";
  import { getHttpClientContext, getL10nOrchestratorContext } from "@/contexts";
  import { formatProcessName, formatTime } from "@/lib/format";

  const MAX_NB_ENTRIES_IN_PAGE = 1_000;

  const client = getHttpClientContext();

  const { t } = getL10nOrchestratorContext();

  const processId = $page.params.processId;

  let nbEntries: number | null = null;
  let beginRange: number | null = null;
  let endRange: number | null = null;
  let processInfo: Process | null = null;
  let logEntries: LogEntry[] = [];
  let loading = true;

  onMount(async () => {
    loading = true;

    try {
      const { count } = await client.nb_process_log_entries({ processId });

      nbEntries = count;

      const { process } = await client.find_process({
        processId,
      });

      if (!process) {
        throw new Error(`Process ${processId} not found`);
      }

      processInfo = process;

      const viewRange = getViewRange(nbEntries, $page.url.searchParams);

      beginRange = viewRange[0];

      endRange = viewRange[1];

      fetchLogEntries(beginRange, endRange);
    } finally {
      loading = false;
    }
  });

  function getViewRange(
    nbEntries: number,
    urlSearchParams: URLSearchParams
  ): [number, number] {
    let begin = 0;
    const beginParam = urlSearchParams.get("begin");
    if (beginParam) {
      begin = Number.parseFloat(beginParam);
    }

    let end = Math.min(nbEntries, MAX_NB_ENTRIES_IN_PAGE);
    const endParam = urlSearchParams.get("end");
    if (endParam) {
      end = Number.parseFloat(endParam);
    }

    return [begin, end];
  }

  async function fetchLogEntries(beginRange: number, endRange: number) {
    if (!processInfo) {
      return;
    }

    loading = true;

    try {
      const response = await client.list_process_log_entries({
        process: processInfo,
        begin: beginRange,
        end: endRange,
      });

      beginRange = response.begin;
      endRange = response.end;
      logEntries = response.entries;
    } finally {
      loading = false;
    }
  }

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

  function buildPaginationHref(begin: number, end: number) {
    return `/log/${processId}?begin=${begin}&end=${end}`;
  }

  $: if (nbEntries !== null) {
    const viewRange = getViewRange(nbEntries, $page.url.searchParams);

    beginRange = viewRange[0];

    endRange = viewRange[1];
  }

  $: if (beginRange !== null && endRange !== null) {
    fetchLogEntries(beginRange, endRange);
  }

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
      <div class="flex items-center h-full">
        {#if nbEntries !== null && nbEntries > MAX_NB_ENTRIES_IN_PAGE}
          <Pagination
            begin={beginRange || 0}
            end={endRange || MAX_NB_ENTRIES_IN_PAGE}
            entriesPerPage={nbEntries}
            maxEntriesPerPage={MAX_NB_ENTRIES_IN_PAGE}
            buildHref={buildPaginationHref}
          />
        {/if}
      </div>
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
                class="truncate"
                title={$t("global-severity-level", { level: value })}
                style={`color:rgb(var(${levelToColorCssVar(value)}));`}
              >
                <L10n id="global-severity-level" variables={{ level: value }} />
              </div>
            {:else if columnName === "timeMs"}
              <div class="flex justify-end w-full" title={formatTime(value)}>
                <div dir="rtl" class="truncate">
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
  :global body {
    overflow-y: scroll;
  }

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
