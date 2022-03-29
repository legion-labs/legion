<script context="module" lang="ts">
  export type ScrollPosition = "start" | "middle" | "end";

  export type ScrollStatus = {
    position: ScrollPosition;
    firstRenderedIndex: number;
    lastRenderedIndex: number;
  };
</script>

<script lang="ts">
  import {
    afterUpdate,
    beforeUpdate,
    createEventDispatcher,
    onMount,
  } from "svelte";
  import { FixedSizeList, styleString } from "svelte-window";
  import type {
    ListOnItemsRenderedProps,
    ListOnScrollProps,
  } from "svelte-window";
  import { Writable, derived, writable } from "svelte/store";

  import { remToPx } from "../lib/html";
  import { debounced, recorded } from "../lib/store";
  import type { LogEntry } from "../types/log";

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
    ([$renderedItems, $scrollInfo]) =>
      $renderedItems && $scrollInfo
        ? { renderedItems: $renderedItems, scrollInfo: $scrollInfo }
        : null
  );

  const scrollPosition = derived(
    scrollStatus,
    ($scrollStatus): ScrollPosition | null => {
      if (!$scrollStatus) {
        return null;
      }

      return $scrollStatus.renderedItems.visibleStartIndex === 0
        ? "start"
        : $scrollStatus.renderedItems.visibleStopIndex === totalCount - 1
        ? "end"
        : "middle";
    }
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

      if (index == null || entries.has(totalCount - index)) {
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

  export let entries: Map<number, LogEntry> = new Map();

  export let totalCount: number;

  export let overscanCount = 2;

  export let noDate = false;

  let rootHeight: number | null = null;

  let headerHeight: number | null = null;

  let fixedSizeList: FixedSizeList | null = null;

  let autoScroll = true;

  $: viewportHeight = (rootHeight || 0) - (headerHeight || 0);

  $: dispatch("requestedIndexChange", $requestedIndex);

  onMount(() => {
    scrollToBottom();
  });

  beforeUpdate(() => {
    if (fixedSizeList) {
      autoScroll =
        !$scrollStatus?.scrollInfo.scrollUpdateWasRequested &&
        $scrollStatus?.scrollInfo.scrollDirection === "backward"
          ? false
          : autoScroll ||
            ($scrollStatus?.scrollInfo.scrollOffset || 0) +
              viewportHeight +
              elementHeight >=
              elementHeight * totalCount;
    }
  });

  afterUpdate(() => {
    if (autoScroll) {
      scrollToBottom();
    }
  });

  export function scrollToBottom() {
    fixedSizeList?.scrollToItem(totalCount);
  }

  function onScroll(newScroll: ListOnScrollProps) {
    $scrollInfo = newScroll;

    if (
      $scrollStatus?.renderedItems &&
      $scrollStatus?.scrollInfo &&
      $scrollPosition
    ) {
      dispatch("scrollStatusChange", {
        firstRenderedIndex: $scrollStatus.renderedItems.visibleStartIndex,
        lastRenderedIndex: $scrollStatus.renderedItems.visibleStopIndex,
        position: $scrollPosition,
      });
    }
  }

  function onItemsRendered(newRenderedItems: ListOnItemsRenderedProps) {
    $renderedItems = newRenderedItems;
  }
</script>

<div class="root" bind:clientHeight={rootHeight}>
  <div class="header" bind:clientHeight={headerHeight}>
    <div class="header-column w-1/12">severity</div>
    {#if !noDate}
      <div class="header-column w-2/12">date</div>
    {/if}
    <div class="header-column w-3/12">target</div>
    <div class="header-column w-6/12">message</div>
  </div>
  <div class="body">
    {#if totalCount > 0}
      <FixedSizeList
        bind:this={fixedSizeList}
        height={viewportHeight}
        itemCount={totalCount}
        itemSize={elementHeight}
        width="100%"
        {overscanCount}
        {onScroll}
        {onItemsRendered}
        let:items
      >
        {#each items as item (item.key)}
          {@const entry = entries.get(item.index)}
          {@const date = noDate ? null : entry?.datetime?.toLocaleString()}

          <div
            class="entry bg-gray-800"
            class:bg-opacity-30={item.index % 2 === 0}
            class:bg-opacity-50={item.index % 2 === 1}
            style={styleString(item.style)}
          >
            {#if entry}
              <div class="flex w-1/12 justify-center entry-column">
                <div
                  class="severity-pill"
                  class:bg-red-500={entry.severity === "error"}
                  class:bg-orange-400={entry.severity === "warn"}
                  class:bg-green-600={entry.severity === "info"}
                  class:bg-gray-600={entry.severity === "trace"}
                  class:bg-blue-500={entry.severity === "debug"}
                >
                  <div class="severity-pill-text">{entry.severity}</div>
                </div>
              </div>
              {#if !noDate}
                <div class="w-2/12 entry-column">{date}</div>
              {/if}
              <div class="w-3/12 entry-column" title={entry.target}>
                {entry.target}
              </div>
              <div class="w-6/12 entry-column" title={entry.message}>
                {entry.message}
              </div>
            {:else}
              <div class="w-1/12 skeleton-column"><div class="skeleton" /></div>
              {#if !noDate}
                <div class="w-2/12 skeleton-column">
                  <div class="skeleton" />
                </div>
              {/if}
              <div class="w-3/12 skeleton-column"><div class="skeleton" /></div>
              <div class="w-6/12 skeleton-column"><div class="skeleton" /></div>
            {/if}
          </div>
        {/each}
      </FixedSizeList>
    {/if}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-full w-full;
  }

  .header {
    @apply uppercase flex flex-row items-center flex-shrink-0 flex-grow-0 h-8 w-full;
  }

  .header-column {
    @apply flex justify-center flex-shrink-0;
  }

  .body {
    @apply flex flex-col h-full flex-shrink-0 flex-grow-0;
  }

  .entry {
    @apply flex flex-row h-full px-2 items-center w-full hover:bg-gray-500;
  }

  .entry-column {
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
