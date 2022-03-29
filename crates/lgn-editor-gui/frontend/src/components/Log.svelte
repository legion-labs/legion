<script lang="ts">
  import { writable } from "svelte/store";

  import type { ScrollStatus } from "@lgn/web-client/src/components/Log.svelte";
  import Log from "@lgn/web-client/src/components/Log.svelte";

  import { buffer, logEntries } from "@/stores/log";

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
