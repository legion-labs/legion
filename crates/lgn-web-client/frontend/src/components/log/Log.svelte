<script lang="ts">
  import { FixedSizeList, styleString } from "svelte-window";
  import type {
    ListOnItemsRenderedProps,
    ListOnScrollProps,
  } from "svelte-window";
  import type { Log } from "../../types/log";
  import { remToPx } from "../../lib/html";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{
    onScroll: ListOnScrollProps;
    onItemsRendered: ListOnItemsRenderedProps;
  }>();

  // Should never be unknown
  const elementHeight = remToPx(2) || 0;

  export let logs: Map<number, Log> = new Map();

  export let totalCount: number;

  export let renderedItems: ListOnItemsRenderedProps | null = null;

  export let scrollInfo: ListOnScrollProps | null = null;

  let rootHeight: number | null = null;

  function onScroll(newScroll: ListOnScrollProps) {
    scrollInfo = newScroll;

    dispatch("onScroll", scrollInfo);
  }

  function onItemsRendered(newRenderedItems: ListOnItemsRenderedProps) {
    renderedItems = newRenderedItems;

    dispatch("onItemsRendered", renderedItems);
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
        overscanCount={100}
        {onScroll}
        {onItemsRendered}
        let:items
      >
        {#each items as item (item.key)}
          {@const log = logs.get(totalCount - item.index)}
          {@const date = log?.timestamp?.toLocaleString()}

          <div
            class="log"
            class:bg-gray-700={item.index % 2 === 0}
            class:bg-gray-800={item.index % 2 === 1}
            style={styleString(item.style)}
          >
            <div class="w-1/12 log-column">
              {totalCount - item.index}{log ? `/${log.id}` : ""}
            </div>
            {#if log}
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
