<script lang="ts">
  import { page } from "$app/stores";
  import { afterUpdate, onMount, tick } from "svelte";
  import { getContext } from "svelte";
  import { get } from "svelte/store";

  import CallGraph from "@/components/CallGraphHierachy/CallGraphHierachy.svelte";
  import DisplayError from "@/components/Misc/DisplayError.svelte";
  import Layout from "@/components/Misc/Layout.svelte";
  import Loader from "@/components/Misc/Loader.svelte";
  import TimeRange from "@/components/Misc/TimeRange.svelte";
  import { TimelineStateManager } from "@/components/Timeline/Stores/TimelineStateManager";
  import type { TimelineStateStore } from "@/components/Timeline/Stores/TimelineStateStore";
  import TimelineProcess from "@/components/Timeline/TimelineProcess.svelte";
  import TimelineAction from "@/components/Timeline/Tools/TimelineAction.svelte";
  import TimelineAxis from "@/components/Timeline/Tools/TimelineAxis.svelte";
  import TimelineMinimap from "@/components/Timeline/Tools/TimelineMinimap.svelte";
  import TimelineSearch from "@/components/Timeline/Tools/TimelineSearch.svelte";
  import { pixelMargin } from "@/components/Timeline/Values/TimelineValues";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { endQueryParam, startQueryParam } from "@/lib/time";

  const processId = $page.params.processId;

  const client = getContext("http-client");
  const threadItemLength = getContext("thread-item-length");

  let stateManager: TimelineStateManager;
  let windowInnerWidth: number;
  let stateStore: TimelineStateStore;
  let canvasHeight: number;
  let scrollHeight: number;
  let scrollTop: number;
  let div: HTMLElement;
  let mainWidth: number;
  let initializationError = "";
  let searching = false;
  let x: number;
  let y: number;
  let callGraphBegin: number | undefined;
  let callGraphEnd: number | undefined;

  $: if (mainWidth && stateStore) {
    stateStore.updateWidth(mainWidth - threadItemLength - pixelMargin);
  }

  $: [x, y] = $stateStore?.viewRange ?? [-Infinity, Infinity];
  $: (x || y) && new Promise(async () => await stateManager?.fetchDynData());

  onMount(async () => {
    loadingStore.reset(10);
    const url = new URLSearchParams($page.url.search);
    const s = url.get(startQueryParam);
    const start = s != null ? Number.parseFloat(s) : null;
    const e = url.get(endQueryParam);
    const end = e != null ? Number.parseFloat(e) : null;
    const canvasWidth = windowInnerWidth - threadItemLength;

    stateManager = new TimelineStateManager(
      $client,
      processId,
      canvasWidth,
      start,
      end
    );
    stateStore = stateManager.state;

    await requestProcessLakehouse(processId);

    try {
      await stateManager.init();
    } catch (error) {
      if (error instanceof Error) {
        initializationError = error.message;
      } else {
        initializationError = "Unknown error.";
      }
      return;
    }

    if (!Object.keys(get(stateStore).blocks).length) {
      initializationError = `Process does not have any block data. Please refresh the page to try again.`;
    }
  });

  async function requestProcessLakehouse(processId: string) {
    if (
      import.meta.env.VITE_LEGION_ANALYTICS_ENABLE_TIMELINE_JIT_LAKEHOUSE ===
      "true"
    ) {
      await $client.build_timeline_tables({
        processId,
      });
    }
  }

  async function onZoom(event: WheelEvent) {
    stateStore.wheelZoom(event);
  }

  function getMouseX(event: MouseEvent) {
    if (event.currentTarget instanceof HTMLElement) {
      const rect = event.currentTarget.getBoundingClientRect();
      return event.clientX - rect.left - threadItemLength;
    }
    return null;
  }

  async function onMouseMove(event: MouseEvent) {
    if (event.buttons === 1) {
      const x = getMouseX(event);
      if (event.shiftKey) {
        if (x) {
          stateStore.updateSelection(x);
        }
      } else {
        if (x) {
          stateStore.applyDrag(x);
        }
        if (event.movementY) {
          div.scrollBy(0, -event.movementY);
          await tick();
        }
      }
    }
  }

  function onMouseDown(event: MouseEvent) {
    if (event.shiftKey) {
      const x = getMouseX(event);
      if (x) {
        stateStore.startSelection(x);
      }
    }
  }

  function onMouseUp(_: MouseEvent) {
    stateStore.stopDrag();
    const selection = $stateStore.currentSelection;
    if (selection) {
      [callGraphBegin, callGraphEnd] = selection;
      setRangeUrl(selection);
    } else {
      callGraphBegin = callGraphEnd = undefined;
    }
  }

  function setRangeUrl(selection: [number, number]) {
    const start = Math.max($stateStore.minMs, selection[0]);
    const end = Math.min($stateStore.maxMs, selection[1]);
    const params = `?${startQueryParam}=${start}&${endQueryParam}=${end}`;
    history.replaceState(null, "", params);
  }

  async function handleKeydown(event: KeyboardEvent) {
    if (event.shiftKey || searching) {
      return;
    }

    switch (event.code) {
      case "Escape":
        onEscape();
        break;
      case "KeyD":
        event.preventDefault();
        stateStore.keyboardTranslate(true);
        break;
      case "KeyA":
        event.preventDefault();
        stateStore.keyboardTranslate(false);
        break;
      case "ArrowUp":
        onVerticalArrow(event, false);
        break;
      case "ArrowDown":
        onVerticalArrow(event, true);
        break;
      case "KeyW":
        stateStore.keyboardZoom(true);
        break;
      case "KeyS":
        stateStore.keyboardZoom(false);
        break;
    }
  }

  function onEscape() {
    stateStore.clearSelection();
    history.replaceState(null, "", "?");
    callGraphBegin = callGraphEnd = undefined;
  }

  async function onVerticalArrow(event: KeyboardEvent, positive: boolean) {
    event.preventDefault();
    if ($stateStore && canvasHeight < scrollHeight) {
      const sign = positive ? 1 : -1;
      div.scrollBy({ top: (sign * (scrollHeight - canvasHeight)) / 10 });
    }
  }

  afterUpdate(() => {
    onScroll(undefined);
  });

  function onScroll(_: UIEvent | undefined) {
    if (div) {
      scrollHeight = div.scrollHeight;
      scrollTop = div.scrollTop;
    }
  }

  async function onMinimapTick(detail: {
    xBegin: number;
    xEnd: number;
    yRatio: number;
  }) {
    internalScrollTop(detail.yRatio * scrollHeight);
    stateStore.setViewRange([detail.xBegin, detail.xEnd]);
  }

  function internalScrollTop(value: number) {
    div.scrollTo({ top: value });
  }

  function onMouseLeave() {
    stateStore.stopDrag();
    if ($stateStore.currentSelection) {
      [callGraphBegin, callGraphEnd] = $stateStore.currentSelection;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} bind:innerWidth={windowInnerWidth} />

<Layout>
  <div slot="content">
    {#if stateStore}
      {#if initializationError}
        <DisplayError error={initializationError} />
      {:else if !$stateStore.ready}
        <Loader />
      {:else}
        <div class="timeline">
          {#if stateManager?.process && $stateStore.ready}
            <div class="pb-1 flex flex-1 flex-row items-center justify-between">
              <TimelineAction
                {processId}
                process={stateManager.process}
                timeRange={$stateStore.currentSelection}
              />
              <TimelineSearch bind:searching />
            </div>
          {/if}
          <div class="main">
            <div
              class="canvas cursor-pointer basis-auto"
              bind:this={div}
              bind:clientHeight={canvasHeight}
              bind:clientWidth={mainWidth}
              on:wheel|preventDefault
              on:scroll={(e) => onScroll(e)}
              on:mousedown|preventDefault={(e) => onMouseDown(e)}
              on:mousemove|preventDefault={(e) => onMouseMove(e)}
              on:mouseleave|preventDefault={(_) => onMouseLeave()}
              on:mouseup|preventDefault={(e) => onMouseUp(e)}
            >
              {#if stateStore}
                {#each $stateStore.processes as p, index (p.processId)}
                  <TimelineProcess
                    process={p}
                    {index}
                    {stateStore}
                    rootStartTime={stateManager.rootStartTime}
                    on:zoom={(e) => onZoom(e.detail)}
                  />
                {/each}
              {/if}
            </div>
            <TimelineMinimap
              {stateStore}
              {canvasHeight}
              {scrollHeight}
              {scrollTop}
              on:zoom={(e) => onZoom(e.detail)}
              on:tick={(e) => onMinimapTick(e.detail)}
            />
            <TimelineAxis {stateStore} />
            <div class="pt-3">
              {#if $stateStore && $stateStore.currentSelection}
                <div class="flex">
                  <div class="min-w-thread-item" />
                  <TimeRange
                    width={$stateStore.canvasWidth}
                    selectionRange={$stateStore.currentSelection}
                    viewRange={$stateStore.viewRange}
                  />
                </div>
              {/if}
            </div>
            {#if callGraphBegin && callGraphEnd}
              <div class="basis-1/5">
                <CallGraph
                  begin={callGraphBegin}
                  end={callGraphEnd}
                  {processId}
                  debug={false}
                  size={250}
                />
              </div>
            {/if}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</Layout>

<style lang="postcss">
  .timeline {
    @apply flex flex-col pt-4 px-2;
  }

  .main {
    /* TODO: Quick hack to prevent the x overflow, find a better fix */
    @apply overflow-x-hidden;

    @apply relative flex flex-col;
    height: calc(100vh - 130px);
  }

  .canvas {
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    @apply gap-y-1;
  }
</style>
