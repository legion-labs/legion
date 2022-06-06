<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { getContext } from "svelte";

  import type { LogEntry } from "@lgn/proto-telemetry/dist/log";
  import { Level } from "@lgn/proto-telemetry/dist/log";
  import type { Process } from "@lgn/proto-telemetry/dist/process";
  import HighlightedText from "@lgn/web-client/src/components/HighlightedText.svelte";
  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";
  import { stringToSafeRegExp } from "@lgn/web-client/src/lib/html";
  import {
    deleteSearchParam,
    setSearchParams,
    updateSearchParams,
  } from "@lgn/web-client/src/lib/navigation";
  import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";

  import L10n from "@/components/Misc/L10n.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import Pagination from "@/components/Misc/Pagination.svelte";
  import SearchInput from "@/components/Misc/SearchInput.svelte";
  import SelectInput from "@/components/Misc/SelectInput.svelte";
  import Table from "@/components/Misc/Table.svelte";
  import { formatProcessName, formatTime } from "@/lib/format";

  const MAX_NB_ENTRIES_IN_PAGE = 1_000;

  const client = getContext("http-client");

  const { t } = getContext(l10nOrchestratorContextKey);
  const columns = [
    { name: "level" as const, width: "7%" },
    { name: "timeMs" as const, width: "8%" },
    { name: "target" as const, width: "10%" },
    { name: "msg" as const, width: "75%" },
  ];

  const levelThresholdOptions = [
    { label: "Error", value: "0" },
    { label: "Warn", value: "1" },
    { label: "Info", value: "2" },
    { label: "Debug", value: "3" },
    { label: "Trace", value: "4" },
  ];

  const processStore = createAsyncStoreOrchestrator<{
    process: Process;
    nbEntries: number;
  }>();

  const logEntriesStore = createAsyncStoreOrchestrator<LogEntry[]>();

  const { data: processInfo, loading: processInfoLoading } = processStore;
  const { data: logEntries, loading: logEntriesLoading } = logEntriesStore;

  function fetchProcessInfo(processId: string) {
    return processStore.run(async () => {
      const { count: nbEntries } = await client.nb_process_log_entries({
        processId,
      });

      const { process } = await client.find_process({
        processId,
      });

      if (!process) {
        throw new Error(`Process ${processId} not found`);
      }

      return { process, nbEntries };
    });
  }

  function fetchLogEntries(
    process: Process,
    nbEntries: number,
    beginRange: number | null,
    endRange: number | null,
    search: string,
    levelThreshold: number | null
  ) {
    return logEntriesStore.run(async () => {
      const cleanSearch = search.trim();

      const response = await client.list_process_log_entries({
        process,
        begin: beginRange ?? 0,
        end: endRange ?? Math.min(nbEntries, MAX_NB_ENTRIES_IN_PAGE),
        search: cleanSearch.length ? cleanSearch : undefined,
        levelThreshold: levelThreshold ?? undefined,
      });

      beginRange = response.begin;
      endRange = response.end;

      return response.entries;
    });
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
    return setSearchParams($page, {
      begin: begin.toString(),
      end: end.toString(),
    });
  }

  function redirectSearch({
    detail: { encodedValue },
  }: CustomEvent<{ encodedValue: string }>) {
    goto(
      updateSearchParams($page, ({ begin, end, search, ...params }) => ({
        ...params,
        ...(encodedValue.length ? { search: encodedValue } : {}),
      })),
      {
        keepfocus: true,
        replaceState: $page.url.searchParams.has("search"),
      }
    );
  }

  function redirectLevelThreshold({ detail: value }: CustomEvent<string>) {
    goto(
      updateSearchParams($page, (allParams) => {
        const { begin, end, "level-threshold": _, ...params } = allParams;

        return {
          ...params,
          ...(value.length ? { "level-threshold": value } : {}),
        };
      }),
      {
        keepfocus: true,
        replaceState: $page.url.searchParams.has("level-threshold"),
      }
    );
  }

  function clearLevelThreshold() {
    goto(deleteSearchParam($page, "level-threshold"));
  }

  function clearSearch() {
    goto(deleteSearchParam($page, "search"));
  }

  $: processId = $page.params.processId;

  // URL Params
  $: beginParam = $page.url.searchParams.get("begin");

  $: endParam = $page.url.searchParams.get("end");

  $: levelThresholdParam = $page.url.searchParams.get("level-threshold");

  $: searchParam = decodeURIComponent(
    $page.url.searchParams.get("search") || ""
  );

  $: levelThreshold =
    levelThresholdParam !== null && !isNaN(+levelThresholdParam)
      ? +levelThresholdParam
      : null;

  // View range
  $: beginRange =
    beginParam !== null && !isNaN(+beginParam) ? +beginParam : null;

  $: endRange = endParam !== null && !isNaN(+endParam) ? +endParam : null;

  // Search related stores
  $: searchValue = searchParam;

  // Fetch the process info on process id change
  $: if (processId !== null) {
    fetchProcessInfo(processId);
  }

  // Fetch the log on params change
  $: if ($processInfo !== null) {
    fetchLogEntries(
      $processInfo.process,
      $processInfo.nbEntries,
      beginRange,
      endRange,
      searchParam,
      levelThreshold
    );
  }

  // UI related derived states
  $: processDescription = $processInfo
    ? `${$processInfo.process.exe} (${$processInfo.process.processId})`
    : null;

  $: formattedProcessName = $processInfo
    ? formatProcessName($processInfo.process)
    : null;

  $: searchPattern = searchParam
    .trim()
    .split(" ")
    .filter(Boolean)
    .map((part) => stringToSafeRegExp(part, "gi"));

  $: formattedTimes =
    $logEntries?.map(({ timeMs }) => formatTime(timeMs)) ?? [];

  $: displayPagination =
    levelThreshold === null &&
    !searchPattern.length &&
    $processInfo &&
    $processInfo.nbEntries !== null &&
    $processInfo.nbEntries > MAX_NB_ENTRIES_IN_PAGE;

  $: loading = $processInfoLoading || $logEntriesLoading;
</script>

<Layout>
  <div slot="header" class="flex space-x-1">
    <SelectInput
      options={levelThresholdOptions}
      value={levelThresholdParam || ""}
      on:clear={clearLevelThreshold}
      on:change={redirectLevelThreshold}
    />
    <SearchInput
      placeholder={$t("log-search")}
      on:clear={clearSearch}
      on:debouncedInput={redirectSearch}
      bind:value={searchValue}
    />
  </div>
  <div
    class="flex flex-row justify-between items-center pl-4 w-full"
    slot="sub-header"
  >
    {#if $processInfo}
      <div class="flex flex-row space-x-2">
        {#if $processInfo.process.parentProcessId}
          <div class="flex flex-row space-x-2 text">
            <a href={`/log/${$processInfo.process.parentProcessId}`}>
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
        {#if displayPagination}
          <Pagination
            begin={beginRange || 0}
            end={endRange || MAX_NB_ENTRIES_IN_PAGE}
            totalEntries={$processInfo.nbEntries}
            entriesPerPage={MAX_NB_ENTRIES_IN_PAGE}
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
        <Table {columns} items={$logEntries || []} sticky={6}>
          <div
            slot="header"
            class="header"
            let:columnName
            title={$t("log-table-column", { columnName })}
          >
            <div class="truncate">
              <L10n id="log-table-column" variables={{ columnName }} />
            </div>
          </div>
          <div slot="cell" class="cell" let:columnName let:item let:index>
            {#if columnName === "level"}
              <div
                class="truncate"
                title={$t("global-severity-level", { level: item[columnName] })}
                style={`color:rgb(var(${levelToColorCssVar(
                  item[columnName]
                )}));`}
              >
                <L10n
                  id="global-severity-level"
                  variables={{ level: item[columnName] }}
                />
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
              <div dir="rtl" class="truncate" title={item[columnName]}>
                {#if searchPattern.length}
                  <HighlightedText
                    pattern={searchPattern}
                    text={item[columnName]}
                  />
                {:else}
                  {item[columnName]}
                {/if}
              </div>
            {:else}
              <div class="break-words w-full">
                {#if searchPattern.length}
                  <HighlightedText
                    pattern={searchPattern}
                    text={item[columnName]}
                  />
                {:else}
                  {item[columnName]}
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
