<script lang="ts">
  import { buffer, logs } from "@/stores/logs";
  import type { ScrollStatus } from "@lgn/web-client/src/components/Log.svelte";
  import Log from "@lgn/web-client/src/components/Log.svelte";
  import { writable } from "svelte/store";

  const scrollStatus = writable<ScrollStatus | null>(null);
</script>

<Log
  {buffer}
  noDate
  logs={$logs}
  totalCount={$logs.size}
  on:scrollStatusChange={({ detail: newsScrollStatus }) =>
    ($scrollStatus = newsScrollStatus)}
/>
