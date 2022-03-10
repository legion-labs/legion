<script lang="ts">
  import type { ScrollStatus } from "@lgn/web-client/src/components/log/Log.svelte";
  import Log from "@lgn/web-client/src/components/log/Log.svelte";
  import type { Log as LogMessage } from "@lgn/web-client/src/types/log";
  import { onMount } from "svelte";
  import type { Writable } from "svelte/store";
  import { derived, writable } from "svelte/store";

  const buffer = 300;

  const totalCount = writable(0);

  const streamedLogs = writable(new Map<number, LogMessage>());

  const staticLogs = writable(new Map<number, LogMessage>());

  const scrollStatus: Writable<ScrollStatus | null> = writable(null);

  const forcePaused = writable(false);

  const paused = derived(
    [forcePaused, scrollStatus],
    ([$forcePaused, $scrollStatus]) =>
      $forcePaused || !!($scrollStatus?.position !== "start")
  );

  const logs = derived(
    [paused, streamedLogs, staticLogs],
    ([paused, streamedLogs, staticLogs]) => {
      let streamedLogsClone = streamedLogs;

      if (paused) {
        streamedLogsClone = new Map(streamedLogs.entries());
      }

      return new Map([...streamedLogsClone, ...staticLogs]);
    }
  );

  let requestedIndex: number | null = null;

  $: if (typeof requestedIndex === "number") {
    fetchStaticLogs(requestedIndex);

    $streamedLogs = new Map();
  }

  onMount(async () => {
    const initStreamedLogsCleanup = await initStreamedLogs();

    await fetchStaticLogs(null);

    return () => {
      // Clears web socket listeners
      initStreamedLogsCleanup();
    };
  });

  async function fetchStaticLogs(index: number | null) {
    let path = `/api/logs?size=${buffer}`;

    if (index != null) {
      path += `&after=${index}`;
    }

    const response = await fetch(`http://localhost:4000${path}`);

    const json = await response.json();

    $totalCount = json.pagination.total_count;

    $staticLogs = new Map(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (json.data as any[]).map(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (log: any) =>
          [
            log.id,
            {
              ...log,
              datetime: new Date(log.datetime),
            },
          ] as [number, LogMessage]
      )
    );
  }

  /**
   * Connects to a web socket that streams logs.
   * Can be run once only
   */
  function initStreamedLogs() {
    return new Promise<() => void>((resolve) => {
      function onOpen() {
        resolve(() => {
          ws.removeEventListener("open", onOpen);
          ws.removeEventListener("message", onMessage);
        });
      }

      async function onMessage(message: MessageEvent<Blob>) {
        const { log, total_count: newTotalCount } = JSON.parse(
          await message.data.text()
        );

        if (!$paused) {
          $totalCount = newTotalCount;
        }

        if (!$streamedLogs.has(log.id)) {
          if ($streamedLogs.size > buffer - 1) {
            const lastItemIndex = Array.from($streamedLogs.keys())[0];

            $streamedLogs.delete(lastItemIndex);
          }

          $streamedLogs = $streamedLogs.set(log.id, {
            ...log,
            datetime: new Date(log.datetime),
          } as LogMessage);
        }
      }

      const ws = new WebSocket("ws://localhost:4000/ws");

      // Making sure the default is properly set to "blob"
      ws.binaryType = "blob";

      ws.addEventListener("open", onOpen);
      ws.addEventListener("message", onMessage);
    });
  }
</script>

<div on:click={() => ($forcePaused = !$forcePaused)}>
  {$paused ? "Unpause" : "Pause"}
</div>

<Log
  {buffer}
  logs={$logs}
  totalCount={$totalCount}
  on:requestedIndexChange={({ detail: newRequestedIndex }) =>
    (requestedIndex = newRequestedIndex)}
  on:scrollStatusChange={({ detail: newsScrollStatus }) =>
    ($scrollStatus = newsScrollStatus)}
/>
