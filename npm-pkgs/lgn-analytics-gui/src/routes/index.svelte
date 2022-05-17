<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  import type { ProcessInstance } from "@lgn/proto-telemetry/dist/analytics";
  import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";

  import L10n from "@/components/Misc/L10n.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import SearchInput from "@/components/Misc/SearchInput.svelte";
  import ProcessItem from "@/components/Process/ProcessItem.svelte";
  import { getHttpClientContext, getL10nOrchestratorContext } from "@/contexts";

  const { t } = getL10nOrchestratorContext();

  const client = getHttpClientContext();

  const processesStore = createAsyncStoreOrchestrator<ProcessInstance[]>();

  const { data: processes, loading } = processesStore;

  function redirectSearch({ detail: value }: CustomEvent<string>) {
    goto(`/${value.length ? `?search=${value}` : ""}`, {
      keepfocus: true,
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

  $: searchParam = $page.url.searchParams.get("search") || "";

  $: searchValue = searchParam;

  $: cleanSearchParam = searchParam.trim();

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
      <div class="process-list">
        <div class="flex flex-col space-y-1">
          <div class="flex flex-row text-content-60">
            <div class="w-8" />
            <div class="w-5/12 xl:w-2/12 truncate hidden md:block">
              <L10n id="process-list-user" />
            </div>
            <div class="w-5/12 xl:w-2/12 truncate">
              <L10n id="process-list-process" />
            </div>
            <div class="w-2/12 truncate hidden xl:block">
              <L10n id="process-list-computer" />
            </div>
            <div class="w-2/12 truncate hidden xl:block">
              <L10n id="process-list-platform" />
            </div>
            <!-- <div class="w-2/12 truncate">Last Activity</div> -->
            <div class="w-2/12 pl-4">
              <L10n id="process-list-start-time" />
            </div>
            <div class="w-24 ml-auto">
              <L10n id="process-list-statistics" />
            </div>
          </div>
          {#each $processes || [] as processInstance, index (processInstance.processInfo?.processId)}
            <ProcessItem
              highlightedPattern={searchParam}
              {processInstance}
              depth={0}
              {index}
              noFold={cleanSearchParam.length > 0}
            />
          {/each}
        </div>
      </div>
    {/if}
  </div>
</Layout>

<style lang="postcss">
  .process-list {
    @apply flex flex-col pt-4 pb-1 px-2 text-sm;
  }
</style>
