<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { onDestroy } from "svelte";

  import type { LogEntry } from "@lgn/proto-telemetry/dist/log";
  import { Level } from "@lgn/proto-telemetry/dist/log";
  import type { Process } from "@lgn/proto-telemetry/dist/process";
  import HighlightedText from "@lgn/web-client/src/components/HighlightedText.svelte";
  import { debounce } from "@lgn/web-client/src/lib/event";
  import { stringToSafeRegExp } from "@lgn/web-client/src/lib/html";
  import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";

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

  const processInfoStore = createAsyncStoreOrchestrator<Process>();
  const logEntriesStore = createAsyncStoreOrchestrator<LogEntry[]>();
  const nbEntrieStore = createAsyncStoreOrchestrator<number>();

  const { data: processInfo, loading: processInfoLoading } = processInfoStore;
  const { data: logEntries, loading: logEntriesLoading } = logEntriesStore;
  const { data: nbEntries, loading: nbEntriesLoading } = nbEntrieStore;

  onMount(async () => {
    await Promise.all([
      nbEntrieStore.run(async () => {
        const { count } = await client.nb_process_log_entries({ processId });

        return count;
      }),

      processInfoStore.run(async () => {
        const { process } = await client.find_process({
          processId,
        });

        if (!process) {
          throw new Error(`Process ${processId} not found`);
        }

        return process;
      }),
    ]);
  });

  onDestroy(() => {
    debouncedInput.clear();
  });

  const debouncedInput = debounce((event) => {
    if (event.target instanceof HTMLInputElement) {
      const cleanValue = event.target.value.trim();

      goto(
        `/log/${processId}${cleanValue.length ? `?search=${cleanValue}` : ""}`,
        { keepfocus: true }
      );
    }
  }, 300);

  async function fetchLogEntries(
    process: Process,
    nbEntries: number,
    beginRange: number | null,
    endRange: number | null,
    search: string
  ) {
    const response = await client.list_process_log_entries({
      process,
      begin: beginRange ?? 0,
      end: endRange ?? Math.min(nbEntries, MAX_NB_ENTRIES_IN_PAGE),
      search: search.length ? search : undefined,
    });

    beginRange = response.begin;
    endRange = response.end;

    return response.entries;
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

  // URL Params
  $: beginParam = $page.url.searchParams.get("begin");

  $: endParam = $page.url.searchParams.get("end");

  $: searchParam = $page.url.searchParams.get("search") || "";

  // View range
  $: beginRange =
    beginParam !== null && !isNaN(+beginParam) ? +beginParam : null;

  $: endRange = endParam !== null && !isNaN(+endParam) ? +endParam : null;

  // Search related stores
  $: searchValue = searchParam;

  // Fetch the log on params change
  $: if ($processInfo !== null && $nbEntries !== null) {
    logEntriesStore.run(() =>
      fetchLogEntries(
        $processInfo!,
        $nbEntries!,
        beginRange,
        endRange,
        searchParam
      )
    );
  }

  // UI related derived states
  $: processDescription = $processInfo
    ? `${$processInfo.exe} (${$processInfo.processId})`
    : null;

  $: formattedProcessName = $processInfo
    ? formatProcessName($processInfo)
    : null;

  $: searchPattern = searchParam
    .split(" ")
    .reduce(
      (acc, part) =>
        part.length ? [...acc, stringToSafeRegExp(part, "gi")] : acc,
      [] as RegExp[]
    );

  $: formattedTimes =
    $logEntries?.map(({ timeMs }) => formatTime(timeMs)) ?? [];

  $: loading = $processInfoLoading || $nbEntriesLoading || $logEntriesLoading;
</script>

<Layout>
  <div slot="header">
    <input
      type="text"
      class="h-8 w-96 text rounded-xs pl-2 bg-default"
      placeholder={$t("log-search")}
      on:keyup={debouncedInput}
      bind:value={searchValue}
    />
  </div>
  <div
    class="flex flex-row justify-between items-center pl-4 w-full"
    slot="sub-header"
  >
    {#if $processInfo}
      <div class="flex flex-row space-x-2">
        {#if $processInfo.parentProcessId}
          <div class="flex flex-row space-x-2 text">
            <a href={`/log/${$processInfo.parentProcessId}`}>
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
        {#if !searchPattern.length && $nbEntries !== null && $nbEntries > MAX_NB_ENTRIES_IN_PAGE}
          <Pagination
            begin={beginRange || 0}
            end={endRange || MAX_NB_ENTRIES_IN_PAGE}
            entriesPerPage={$nbEntries}
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
          items={$logEntries || []}
        >
          <div
            slot="header"
            class="header"
            let:columnName
            title={$t("log-parent-table-column", { columnName })}
          >
            <div class="truncate">
              <L10n id="log-parent-table-column" variables={{ columnName }} />
            </div>
          </div>
          <div slot="cell" class="cell" let:columnName let:value let:index>
            {#if columnName === "level"}
              <div
                class="truncate"
                title={$t("global-severity-level", { level: value })}
                style={`color:rgb(var(${levelToColorCssVar(value)}));`}
              >
                <L10n id="global-severity-level" variables={{ level: value }} />
              </div>
            {:else if columnName === "timeMs"}
              <div
                class="flex justify-end w-full"
                title={formattedTimes[index]}
              >
                <div dir="rtl" class="truncate">
                  {formattedTimes[index]}
                </div>
              </div>
            {:else if columnName === "target"}
              <div dir="rtl" class="truncate" title={value}>
                {#if searchPattern.length}
                  <HighlightedText pattern={searchPattern} text={value} />
                {:else}
                  {value}
                {/if}
              </div>
            {:else}
              <div class="break-words w-full">
                {#if searchPattern.length}
                  <HighlightedText pattern={searchPattern} text={value} />
                {:else}
                  {value}
                {/if}
              </div>
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

  .header {
    @apply flex flex-row items-center h-full headline text-base;
  }

  .cell {
    @apply flex flex-row h-full py-0.5 headline;
  }
</style>
