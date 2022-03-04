<script lang="ts">
  import { EMPTY, BehaviorSubject, of, from, iif } from "rxjs";
  import type { Observable } from "rxjs";
  import { fromFetch } from "rxjs/fetch";
  import { webSocket } from "rxjs/webSocket";
  import {
    concatAll,
    map,
    shareReplay,
    mergeWith,
    mergeMap,
    withLatestFrom,
    debounceTime,
    distinctUntilChanged,
    tap,
    filter,
    catchError,
    startWith,
    bufferCount,
    switchMap,
  } from "rxjs/operators";
  import type {
    ListOnItemsRenderedProps,
    ListOnScrollProps,
  } from "svelte-window";
  import Log from "@lgn/web-client/src/components/log/Log.svelte";
  import type { Log as LogMessage } from "@lgn/web-client/src/types/log";
  import { onMount } from "svelte";

  const buffer = 300;

  const renderedItems = new BehaviorSubject<ListOnItemsRenderedProps | null>(
    null
  );

  const scrollInfo = new BehaviorSubject<ListOnScrollProps | null>(null);

  const logs = new BehaviorSubject<Map<number, LogMessage>>(new Map());

  const totalCount = new BehaviorSubject(0);

  $: currentTopIndex = $scrollInfo?.scrollOffset || 0;

  $: paused = currentTopIndex !== 0;

  // TODO: Move to Log.svelte
  /** Get requested index based on the user interaction with the logs viewport */
  function getRequestedIndex(): Observable<number> {
    return scrollInfo.pipe(
      withLatestFrom(renderedItems),
      filter(([scrollInfo, renderedItems]) => !!(scrollInfo && renderedItems)),
      debounceTime(50),
      distinctUntilChanged(
        (prev, curr) =>
          prev[1]?.overscanStartIndex === curr[1]?.overscanStartIndex ||
          prev[1]?.overscanStopIndex === curr[1]?.overscanStopIndex
      ),
      withLatestFrom(logs, totalCount),
      mergeMap(([[scrollInfo, renderedItems], logs, totalCount]) => {
        if (!scrollInfo || !renderedItems) {
          return EMPTY;
        }

        let index: number | null = null;

        if (scrollInfo.scrollDirection === "backward") {
          index = renderedItems.overscanStartIndex;
        }

        if (scrollInfo.scrollDirection === "forward") {
          index = renderedItems.overscanStopIndex;
        }

        if (index == null || logs.has(totalCount - index)) {
          return EMPTY;
        }

        index = Math.round(index - buffer / 2);

        if (index < 0) {
          index = 0;
        }

        if (index > totalCount) {
          index = totalCount - buffer;
        }

        return of(index);
      })
    );
  }

  function getStaticLogsSource(): Observable<{
    logs: LogMessage[];
    totalCount: number;
  }> {
    return getRequestedIndex().pipe(
      startWith(undefined),
      map((index) => {
        let path = `/api/logs?size=${buffer}`;

        if (index != null) {
          path += `&after=${index}`;
        }

        return path;
      }),
      mergeMap((path) =>
        fromFetch(`http://localhost:4000${path}`).pipe(
          mergeMap((response) => response.json()),
          catchError(() => EMPTY)
        )
      ),
      filter(Boolean),
      mergeMap((json) => {
        const logs =
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (json.data as any[]).map(
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (log: any): LogMessage => ({
              ...log,
              timestamp: new Date(log.timestamp),
            })
          );

        const totalCount = json.pagination.total_count;

        return of({ logs, totalCount });
      })
    );
  }

  /**
   * Connects to a web socket that streams logs.
   * Can be run once only
   */
  function getDynamicLogsSource(): Observable<{
    logs: LogMessage[];
    totalCount: number;
  }> {
    return webSocket({
      url: "ws://localhost:4000/ws",
      binaryType: "blob",
      deserializer: (event) => (event.data as Blob).text(),
    }).pipe(
      catchError(() => EMPTY),
      // Make it "cold", preventing subscriptions to trigger subsequent connections
      shareReplay(),
      // Resolves the promise gotten from turning the Blob into a string
      concatAll(),
      // TODO: Remove to stream web socket logs
      // switchMap(() => EMPTY),
      // Parse the string as a valid logs/totalCount object
      map((message) => {
        const { log, total_count: totalCount } = JSON.parse(message);

        return {
          totalCount,
          logs: [{ ...log, timestamp: new Date(log.timestamp) } as LogMessage],
        };
      }),
      switchMap((logs) => iif(() => paused, EMPTY, of(logs)))
    );
  }

  onMount(() => {
    const subscription = getDynamicLogsSource()
      .pipe(
        mergeWith(getStaticLogsSource()),
        tap(({ totalCount: newTotalCount }) => totalCount.next(newTotalCount)),
        mergeMap(({ logs }) => from(logs)),
        map((log) => [log.id, log] as const),
        bufferCount(buffer, 1),
        map((logs) => new Map(logs))
      )
      .subscribe((value) => logs.next(value));

    return () => {
      subscription.unsubscribe();
    };
  });
</script>

<div on:click={() => (paused = !paused)}>{paused ? "Unpause" : "Pause"}</div>

<Log
  logs={$logs}
  totalCount={$totalCount}
  on:onItemsRendered={({ detail: newRenderedItems }) =>
    renderedItems.next(newRenderedItems)}
  on:onScroll={({ detail: newScroll }) => scrollInfo.next(newScroll)}
/>
