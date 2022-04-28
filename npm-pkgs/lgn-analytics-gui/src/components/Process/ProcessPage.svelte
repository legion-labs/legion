<script lang="ts">
  import { onMount } from "svelte";

  import type {
    PerformanceAnalyticsClientImpl,
    ProcessInstance,
  } from "@lgn/proto-telemetry/dist/analytics";

  import { makeGrpcClient } from "@/lib/client";

  import Loader from "../Misc/Loader.svelte";
  import ProcessItem from "./ProcessItem.svelte";

  let processes: ProcessInstance[] = [];
  let client: PerformanceAnalyticsClientImpl | null = null;
  let loading = true;

  onMount(async () => {
    client = makeGrpcClient();
    const result = await client
      .list_recent_processes({
        parentProcessId: "",
      })
      .finally(() => (loading = false));
    processes = result.processes.filter((p) => p.lastActivity);
  });

  async function onSearchChange(
    evt: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    if (!client) {
      return;
    }
    const searchString = evt.currentTarget.value;
    const response = await client.search_processes({ search: searchString });
    processes = response.processes.filter((p) => p.lastActivity);
  }
</script>

<Loader {loading}>
  <div slot="body" class="text-headline text-sm">
    <div class="text-center pb-6">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        autofocus
        type="text"
        class="h-8 w-96 placeholder-text rounded-sm pl-2 bg-surface"
        placeholder="Search process..."
        on:input={onSearchChange}
      />
    </div>
    <div class="flex flex-col gap-y-2 text-sm">
      <div class="flex flex-row text-content-60">
        <div class="w-8" />
        <div class="w-4/12 xl:w-2/12 truncate hidden md:block">User</div>
        <div class="w-4/12 xl:w-2/12 truncate">Process</div>
        <div class="w-2/12 truncate hidden xl:block">Computer</div>
        <div class="w-2/12 truncate hidden xl:block">Platform</div>
        <div class="w-2/12 truncate">Last Activity</div>
        <div class="w-2/12 pl-4 hidden xl:block">Start Time</div>
        <div class="w-24 ml-auto">Statistics</div>
      </div>
      {#each processes as processInstance, index (processInstance.processInfo?.processId)}
        <ProcessItem {processInstance} depth={0} {index} />
      {/each}
    </div>
  </div>
</Loader>
