<script lang="ts">
  import { TimelineStateManager } from "@/lib/Timeline/TimelineStateManager";
  import { afterUpdate, onMount } from "svelte";
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { BarLoader } from "svelte-loading-spinners";
  import {
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
  } from "@/lib/time_range_selection";
  import TimelineAction from "./TimelineAction.svelte";
  import TimelineDebug from "./TimelineDebug.svelte";
  import TimelineProcess from "./TimelineProcess.svelte";
  import TimelineRange from "./TimelineRange.svelte";
  import TimelineSearch from "./TimelineSearch.svelte";
  export let processId: string;
  import { useLocation } from "svelte-navigator";
  import TimelineMinimap from "./TimelineMinimap.svelte";
  import { threadItemLength } from "@/lib/Timeline/TimelineValues";
  import TimelineAxis from "./TimelineAxis.svelte";

  const gap = 4;
  const location = useLocation();
  const startParam = "begin";
  const endParam = "end";

  type PanState = {
    beginMouseX: number;
    beginMouseY: number;
    viewRange: [number, number];
  };

  let stateManager: TimelineStateManager;
  let windowInnerWidth: number;
  let stateStore: TimelineStateStore;
  let panState: PanState | undefined = undefined;
  let canvasHeight: number;
  let scrollHeight: number;
  let scrollTop: number;
  let div: HTMLElement;
  let mainWidth: number;

  $: if (mainWidth && stateStore) {
    stateStore.update((s) => {
      s.canvasWidth = mainWidth - threadItemLength - gap;
      return s;
    });
  }

  $: style = `display:${$stateStore?.ready ? "block" : "none"}`;

  onMount(async () => {
    loadingStore.reset();
    const url = new URLSearchParams($location.search);
    const s = url.get(startParam);
    const start = s != null ? Number.parseFloat(s) : null;
    const e = url.get(endParam);
    const end = e != null ? Number.parseFloat(e) : null;
    const canvasWidth = windowInnerWidth - threadItemLength;
    stateManager = new TimelineStateManager(processId, canvasWidth, start, end);
    stateStore = stateManager.state;
    await stateManager.init();
  });

  async function onZoom(event: WheelEvent) {
    stateStore.update((s) => {
      s.setViewRangeFromWheel(s.getViewRange(), event);
      return s;
    });

    await stateManager.fetchDynData();
  }

  function isValidEvent(event: MouseEvent) {
    return (
      event.target instanceof HTMLCanvasElement ||
      (event.target instanceof Element &&
        event.target.classList.contains("timeline-item"))
    );
  }

  async function onMouseMove(event: MouseEvent) {
    if (
      isValidEvent(event) &&
      RangeSelectionOnMouseMove(
        event,
        $stateStore.selectionState,
        $stateStore.canvasWidth,
        $stateStore.getViewRange()
      )
    ) {
      if (
        $stateStore.currentSelection != $stateStore.selectionState.selectedRange
      ) {
        stateStore.update((s) => {
          s.currentSelection = s.selectionState.selectedRange;
          return s;
        });
      }
      return;
    }

    if (event.buttons !== 1) {
      panState = undefined;
    } else if (!event.shiftKey) {
      if (isValidEvent(event)) {
        await applyDrag(event.offsetX, event.offsetY, event.movementY);
      }
    }
  }

  async function applyDrag(
    offsetX: number,
    offsetY: number,
    movementY: number
  ) {
    if (!panState) {
      panState = {
        beginMouseX: offsetX,
        beginMouseY: offsetY,
        viewRange: $stateStore.getViewRange(),
      };
    }

    const factor =
      (panState.viewRange[1] - panState.viewRange[0]) / $stateStore.canvasWidth;
    const offsetMs = factor * (panState.beginMouseX - offsetX);

    if (movementY) {
      div.scrollBy(0, -movementY);
    }

    stateStore.update((s) => {
      if (panState) {
        s.setViewRange([
          panState.viewRange[0] + offsetMs,
          panState.viewRange[1] + offsetMs,
        ]);
      }
      return s;
    });
    await stateManager.fetchDynData();
  }

  function onMouseDown(event: MouseEvent) {
    if (RangeSelectionOnMouseDown(event, $stateStore.selectionState)) {
      stateStore.update((s) => {
        s.currentSelection = s.selectionState.selectedRange;
        return s;
      });
    }
  }

  function onMouseUp(_: MouseEvent) {
    const selection = $stateStore.currentSelection;
    if (selection) {
      setRangeUrl(selection);
    }
  }

  function setRangeUrl(selection: [number, number]) {
    const start = Math.max($stateStore.minMs, selection[0]);
    const end = Math.min($stateStore.maxMs, selection[1]);
    const params = `?${startParam}=${start}&${endParam}=${end}`;
    history.replaceState(null, "", params);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.code == "Escape" && $stateStore.currentSelection) {
      stateStore.update((s) => {
        s.currentSelection = undefined;
        s.selectionState = NewSelectionState();
        setRangeUrl([s.minMs, s.maxMs]);
        return s;
      });
    }
  }

  afterUpdate(() => {
    onScroll(undefined);
  });

  function onScroll(_: UIEvent | undefined) {
    scrollHeight = div.scrollHeight;
    scrollTop = div.scrollTop;
  }

  async function onMinimapTick(detail: {
    xBegin: number;
    xEnd: number;
    yRatio: number;
  }) {
    panState = undefined;
    div.scrollTo({ top: detail.yRatio * scrollHeight });
    stateStore.update((s) => {
      s.setViewRange([detail.xBegin, detail.xEnd]);
      return s;
    });
    await stateManager.fetchDynData();
  }
</script>

<svelte:window on:keydown={handleKeydown} bind:innerWidth={windowInnerWidth} />

{#if stateStore && !$stateStore.ready}
  <div class="flex items-center justify-center loader">
    <BarLoader />
  </div>
{/if}

<div {style} class="main">
  {#if stateManager?.process && $stateStore.ready}
    <div class="pb-1 flex flex-row items-center justify-between">
      <TimelineAction
        {processId}
        process={stateManager.process}
        timeRange={$stateStore.currentSelection}
      />
      <TimelineSearch />
    </div>
  {/if}

  <div
    class="canvas"
    bind:this={div}
    bind:clientHeight={canvasHeight}
    bind:clientWidth={mainWidth}
    on:scroll={(e) => onScroll(e)}
    on:mousedown|preventDefault={(e) => onMouseDown(e)}
    on:mousemove|preventDefault={(e) => onMouseMove(e)}
    on:mouseup|preventDefault={(e) => onMouseUp(e)}
  >
    {#if stateStore}
      {#each $stateStore.processes as p}
        <TimelineProcess
          process={p}
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
  <div class="range">
    <TimelineRange {stateStore} />
  </div>
</div>

{#if stateManager?.process && $stateStore.ready}
  <div
    class="flex flex-row-reverse justify-between items-center pt-1 h-7 detail"
  >
    <TimelineDebug store={stateStore} />
  </div>
{/if}

<style lang="postcss">
  .main {
    overflow-x: hidden;
    overflow-y: hidden;
    position: relative;
  }

  .canvas {
    max-height: calc(100vh - 150px);
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    @apply gap-y-1;
  }

  .loader {
    height: 90vh;
  }

  .range {
    margin-top: 12px;
  }

  ::-webkit-scrollbar {
    width: 20px;
  }

  ::-webkit-scrollbar-track {
    background-color: transparent;
  }

  ::-webkit-scrollbar-thumb {
    background-color: #bac1c4;
    border-radius: 20px;
    border: 6px solid transparent;
    background-clip: content-box;
  }

  ::-webkit-scrollbar-thumb:hover {
    background-color: #8c9b9e;
  }
</style>
