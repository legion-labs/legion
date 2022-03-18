<script lang="ts">
  import { buffer, logEntries } from "@/stores/log";
  import type { ScrollStatus } from "@lgn/web-client/src/components/Log.svelte";
  import Log from "@lgn/web-client/src/components/Log.svelte";
  import { writable } from "svelte/store";

  const scrollStatus = writable<ScrollStatus | null>(null);
</script>

<Log
  {buffer}
  noDate
  entries={$logEntries}
  totalCount={$logEntries.size}
  on:scrollStatusChange={({ detail: newsScrollStatus }) =>
    ($scrollStatus = newsScrollStatus)}
/>
