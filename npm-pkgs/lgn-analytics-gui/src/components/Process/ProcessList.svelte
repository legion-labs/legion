<script lang="ts">
  import { getContext } from "svelte";

  import type { ProcessInstance } from "@lgn/proto-telemetry/dist/analytics";
  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";

  import L10n from "../Misc/L10n.svelte";
  import Table from "../Misc/Table.svelte";
  import ProcessItem from "./ProcessItem.svelte";

  const { t } = getContext(l10nOrchestratorContextKey);

  const columns = [
    { name: "process" as const, width: "20%" },
    { name: "user" as const, width: "15%" },
    { name: "computer" as const, width: "10%" },
    { name: "platform" as const, width: "10%" },
    { name: "start-time" as const, width: "35%" },
    { name: "statistics" as const, width: "10%" },
  ];

  export let processes: ProcessInstance[];

  export let highlightedPattern: string | RegExp | undefined = undefined;

  export let viewMode: "all" | "search";

  export let depth: number = 0;

  export let headless = false;
</script>

{#if headless}
  <Table
    customKey={({ processInfo }, index) => processInfo?.processId ?? index}
    {columns}
    items={processes}
    sticky={3.5}
  >
    <div slot="row" let:item let:normalizedColumns>
      <ProcessItem
        {depth}
        columns={normalizedColumns}
        {viewMode}
        {highlightedPattern}
        processInstance={item}
        noFold={viewMode === "search"}
      />
    </div>
  </Table>
{:else}
  <Table
    customKey={({ processInfo }, index) => processInfo?.processId ?? index}
    {columns}
    items={processes}
    sticky={3.5}
  >
    <div
      slot="header"
      class="header"
      let:columnName
      title={$t("process-list-table-column", { columnName })}
    >
      {#if !headless}
        <div class="truncate">
          <L10n id="process-list-table-column" variables={{ columnName }} />
        </div>
      {/if}
    </div>
    <div slot="row" let:item let:normalizedColumns>
      <ProcessItem
        {depth}
        columns={normalizedColumns}
        {viewMode}
        {highlightedPattern}
        processInstance={item}
        noFold={viewMode === "search"}
      />
    </div>
  </Table>
{/if}

<style lang="postcss">
  .header {
    @apply flex flex-row items-center h-full headline text-base;
  }
</style>
