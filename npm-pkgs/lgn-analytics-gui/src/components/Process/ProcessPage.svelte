<script lang="ts">
  import { getContext, onMount } from "svelte";

  import type {
    PerformanceAnalyticsClientImpl,
    ProcessInstance,
  } from "@lgn/proto-telemetry/dist/analytics";
  import L10n from "@lgn/web-client/src/components/L10n.svelte";
  import type { L10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";

  import { l10nOrchestratorContextKey } from "@/constants";
  import { makeGrpcClient } from "@/lib/client";

  import Loader from "../Misc/Loader.svelte";
  import ProcessItem from "./ProcessItem.svelte";

  const { t } = getContext<L10nOrchestrator>(l10nOrchestratorContextKey);

  let processes: ProcessInstance[] = [];
  let client: PerformanceAnalyticsClientImpl | null = null;
  let loading = true;

  onMount(async () => {
    client = makeGrpcClient();
    const result = await client
      .list_recent_processes({ parentProcessId: undefined })
      .finally(() => (loading = false));
    processes = result.processes;
  });

  async function onSearchChange(
    evt: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    if (!client) {
      return;
    }
    const searchString = evt.currentTarget.value;
    const response = await client.search_processes({ search: searchString });
    processes = response.processes;
  }
</script>

<Loader {loading}>
  <div slot="body" class="headline text-sm">
    <div class="text-center pb-6">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        autofocus
        type="text"
        class="h-8 w-96 placeholder rounded-sm pl-2 surface"
        placeholder={$t("process-list-search")}
        on:input={onSearchChange}
      />
    </div>
    <div class="flex flex-col gap-y-2 text-sm">
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
      {#each processes as processInstance, index (processInstance.processInfo?.processId)}
        <ProcessItem {processInstance} depth={0} {index} />
      {/each}
    </div>
  </div>
</Loader>
