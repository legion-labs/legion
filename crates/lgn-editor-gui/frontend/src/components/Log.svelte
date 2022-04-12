<script lang="ts">
  import { writable } from "svelte/store";

  import type { ScrollStatus } from "@lgn/web-client/src/components/Log.svelte";
  import Log from "@lgn/web-client/src/components/Log.svelte";

  import { buffer, logEntries } from "@/stores/log";

  const scrollStatus = writable<ScrollStatus | null>(null);

  function onScrollStatusChange({
    detail: newsScrollStatus,
  }: CustomEvent<ScrollStatus | null>) {
    $scrollStatus = newsScrollStatus;
  }
</script>

<Log
  {buffer}
  noDate
  entries={$logEntries}
  totalCount={$logEntries.size}
  on:scrollStatusChange={onScrollStatusChange}
/>
