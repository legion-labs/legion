<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  import type { ProcessInstance } from "@lgn/proto-telemetry/dist/analytics";
  import { stringToSafeRegExp } from "@lgn/web-client/src/lib/html";
  import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";

  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import SearchInput from "@/components/Misc/SearchInput.svelte";
  import ProcessList from "@/components/Process/ProcessList.svelte";
  import { getHttpClientContext, getL10nOrchestratorContext } from "@/contexts";

  const { t } = getL10nOrchestratorContext();

  const client = getHttpClientContext();

  const processesStore = createAsyncStoreOrchestrator<ProcessInstance[]>();

  const { data: processes, loading } = processesStore;

  function redirectSearch({
    detail: { encodedValue },
  }: CustomEvent<{ encodedValue: string }>) {
    goto(`/${encodedValue.length ? `?search=${encodedValue}` : ""}`, {
      keepfocus: true,
      replaceState: $page.url.searchParams.has("search"),
    });
  }

  function search(search: string) {
    return processesStore.run(async () => {
      const response = search.length
        ? await client.search_processes({
            search,
          })
        : await client.list_recent_processes({ parentProcessId: undefined });

      return response.processes;
    });
  }

  $: searchParam = decodeURIComponent(
    $page.url.searchParams.get("search") || ""
  );

  $: searchValue = searchParam;

  $: cleanSearchParam = searchParam.trim();

  $: highlightedPattern = cleanSearchParam.length
    ? stringToSafeRegExp(cleanSearchParam, "gi")
    : undefined;

  $: search(cleanSearchParam);
</script>

<Layout>
  <div slot="header">
    <div class="flex flex-row space-x-0.5">
      <SearchInput
        autofocus
        placeholder={$t("process-list-search")}
        on:clear={() => goto("/")}
        on:debouncedInput={redirectSearch}
        bind:value={searchValue}
      />
    </div>
  </div>
  <div slot="content">
    {#if $loading}
      <Loader />
    {:else}
      <ProcessList
        processes={$processes || []}
        {highlightedPattern}
        viewMode={cleanSearchParam.length ? "search" : "all"}
      />
    {/if}
  </div>
</Layout>
