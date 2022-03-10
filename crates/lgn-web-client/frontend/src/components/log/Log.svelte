<script context="module" lang="ts">
  export type ScrollPosition = "start" | "middle" | "end";

  export type ScrollStatus = {
    position: ScrollPosition;
    firstRenderedIndex: number;
    lastRenderedIndex: number;
  };
</script>

<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { derived, Writable, writable } from "svelte/store";
  import type { Readable } from "svelte/store";
  import { FixedSizeList, styleString } from "svelte-window";
  import type {
    ListOnItemsRenderedProps,
    ListOnScrollProps,
  } from "svelte-window";
  import type { Log } from "../../types/log";
  import { remToPx } from "../../lib/html";
  import { debounced, recorded } from "../../lib/store";

  const dispatch = createEventDispatcher<{
    /**
     * Dispatched when the component doesn't know what to display anymore.
     *
     * The requested index is calculated based on the scroll direction and the total count.
     */
    requestedIndexChange: number | null;
    /** Always dispatched on scroll */
    scrollStatusChange: ScrollStatus | null;
  }>();

  // Should never be unknown
  const elementHeight = remToPx(2) || 0;

  const renderedItems: Writable<ListOnItemsRenderedProps | null> =
    writable(null);

  const scrollInfo: Writable<ListOnScrollProps | null> = writable(null);

  const scrollStatus = derived(
    [renderedItems, scrollInfo],
    ([renderedItems, scrollInfo]) =>
      renderedItems && scrollInfo ? { renderedItems, scrollInfo } : null
  );

  const requestedIndex = derived(
    recorded(debounced(scrollStatus, 50)),
    ($scrollStatus, set: (value: number | null) => void) => {
      if (!$scrollStatus.curr || !$scrollStatus.prev) {
        return;
      }

      const scrollDirection = $scrollStatus.curr.scrollInfo.scrollDirection;

      const prevOverscanStartIndex =
        $scrollStatus.prev.renderedItems.overscanStartIndex;
      const prevOverscanStopIndex =
        $scrollStatus.prev.renderedItems.overscanStopIndex;
      const currOverscanStartIndex =
        $scrollStatus.curr.renderedItems.overscanStartIndex;
      const currOverscanStopIndex =
        $scrollStatus.curr.renderedItems.overscanStopIndex;

      if (
        (prevOverscanStartIndex === currOverscanStartIndex &&
          scrollDirection === "backward") ||
        (prevOverscanStopIndex === currOverscanStopIndex &&
          scrollDirection === "forward")
      ) {
        return;
      }

      let index: number | null = null;

      if (scrollDirection === "backward") {
        index = currOverscanStartIndex;
      }

      if (scrollDirection === "forward") {
        index = currOverscanStopIndex;
      }

      // Was staticlogs.has()
      if (index == null || logs.has(totalCount - index)) {
        return;
      }

      index = Math.round(index - buffer / 2);

      if (index < 0) {
        index = 0;
      }

      if (index > totalCount) {
        index = totalCount - buffer;
      }

      set(index);
    },
    null
  );

  export let buffer: number;

  export let logs: Map<number, Log> = new Map();

  export let totalCount: number;

  let rootHeight: number | null = null;

  $: dispatch("requestedIndexChange", $requestedIndex);

  function onScroll(newScroll: ListOnScrollProps) {
    $scrollInfo = newScroll;

    if ($scrollStatus?.renderedItems && $scrollStatus?.scrollInfo) {
      dispatch("scrollStatusChange", {
        firstRenderedIndex: $scrollStatus.renderedItems.visibleStartIndex,
        lastRenderedIndex: $scrollStatus.renderedItems.visibleStopIndex,
        position:
          $scrollStatus.renderedItems.visibleStartIndex === 0
            ? "start"
            : $scrollStatus.renderedItems.visibleStopIndex === totalCount - 1
            ? "end"
            : "middle",
      });
    }
  }

  function onItemsRendered(newRenderedItems: ListOnItemsRenderedProps) {
    $renderedItems = newRenderedItems;
  }
</script>

<div class="root" bind:clientHeight={rootHeight}>
  <div class="header">
    <div class="header-column w-1/12">#</div>
    <div class="header-column w-1/6">date</div>
    <div class="header-column w-1/2">name</div>
    <div class="header-column w-1/6">target</div>
    <div class="header-column w-1/12">severity</div>
  </div>
  <div class="body">
    {#if totalCount > 0}
      <FixedSizeList
        height={rootHeight}
        itemCount={totalCount}
        itemSize={elementHeight}
        width="100%"
        overscanCount={20}
        {onScroll}
        {onItemsRendered}
        let:items
      >
        {#each items as item (item.key)}
          {@const log = logs.get(totalCount - item.index)}
          {@const date = log?.datetime?.toLocaleString()}

          <div
            class="log"
            class:bg-gray-700={item.index % 2 === 0}
            class:bg-gray-800={item.index % 2 === 1}
            style={styleString(item.style)}
          >
            {#if log}
              <div class="w-1/12 log-column">
                {log ? log.id : ""}
              </div>
              <div class="w-1/6 log-column">{date}</div>
              <div class="w-1/2 log-column" title={log.message}>
                {log.message}
              </div>
              <div class="w-1/6 log-column" title={log.target}>
                {log.target}
              </div>
              <div class="flex w-1/12 log-column justify-center">
                <div
                  class="severity-pill"
                  class:bg-red-500={log.severity === "error"}
                  class:bg-orange-400={log.severity === "warn"}
                  class:bg-green-600={log.severity === "info"}
                  class:bg-gray-600={log.severity === "trace"}
                  class:bg-blue-500={log.severity === "debug"}
                >
                  <div class="severity-pill-text">{log.severity}</div>
                </div>
              </div>
            {:else}
              <div class="w-1/12 skeleton-column"><div class="skeleton" /></div>
              <div class="w-1/6 skeleton-column"><div class="skeleton" /></div>
              <div class="w-1/2 skeleton-column"><div class="skeleton" /></div>
              <div class="w-1/6 skeleton-column"><div class="skeleton" /></div>
              <div class="w-1/12 skeleton-column"><div class="skeleton" /></div>
            {/if}
          </div>
        {/each}
      </FixedSizeList>
    {/if}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-[800px] w-full;
  }

  .header {
    @apply uppercase flex flex-row items-center h-8 w-full;
  }

  .header-column {
    @apply flex justify-center flex-shrink-0;
  }

  .body {
    @apply flex flex-col;
  }

  .log {
    @apply flex flex-row h-full px-2 items-center w-full hover:bg-gray-500 cursor-pointer;
  }

  .log-column {
    @apply px-2 flex-shrink-0 truncate;
  }

  .severity-pill {
    @apply px-2 my-1 text-sm text-black font-bold bg-opacity-70 -skew-x-12;
  }

  .severity-pill-text {
    @apply skew-x-12;
  }

  .skeleton-column {
    @apply animate-pulse px-2 flex-shrink-0;
  }

  .skeleton {
    @apply h-2 w-12 rounded-full bg-white bg-opacity-25;
  }
</style>
