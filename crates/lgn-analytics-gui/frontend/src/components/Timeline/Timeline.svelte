<script lang="ts">
  import { afterUpdate, onMount, tick } from "svelte";
  import { useLocation } from "svelte-navigator";
  import { get } from "svelte/store";

  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { TimelineStateManager } from "@/lib/Timeline/TimelineStateManager";
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import {
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
  } from "@/lib/time_range_selection";

  import Loader from "../Misc/Loader.svelte";
  import TimelineProcess from "./TimelineProcess.svelte";
  import TimelineAction from "./Tools/TimelineAction.svelte";
  import TimelineAxis from "./Tools/TimelineAxis.svelte";
  import TimelineMinimap from "./Tools/TimelineMinimap.svelte";
  import TimelineRange from "./Tools/TimelineRange.svelte";
  import TimelineSearch from "./Tools/TimelineSearch.svelte";
  import { pixelMargin, threadItemLength } from "./Values/TimelineValues";

  export let processId: string;

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
  let initializationError = "";

  $: if (mainWidth && stateStore) {
    stateStore.update((s) => {
      s.canvasWidth = mainWidth - threadItemLength - pixelMargin;
      return s;
    });
  }

  onMount(async () => {
    loadingStore.reset(10);
    const url = new URLSearchParams($location.search);
    const s = url.get(startParam);
    const start = s != null ? Number.parseFloat(s) : null;
    const e = url.get(endParam);
    const end = e != null ? Number.parseFloat(e) : null;
    const canvasWidth = windowInnerWidth - threadItemLength;

    stateManager = new TimelineStateManager(processId, canvasWidth, start, end);
    stateStore = stateManager.state;

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
      await tick();
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
    switch (event.code) {
      case "Escape":
        onEscape();
        break;
      case "ArrowRight":
      case "ArrowLeft":
        onHorizontalArrow(event);
        break;
      case "ArrowUp":
      case "ArrowDown":
        onVerticalArrow(event);
        break;
    }
  }

  function onEscape() {
    if ($stateStore.currentSelection) {
      stateStore.update((s) => {
        s.currentSelection = undefined;
        s.selectionState = NewSelectionState();
        setRangeUrl([s.minMs, s.maxMs]);
        return s;
      });
    }
  }

  function onHorizontalArrow(event: KeyboardEvent) {
    event.preventDefault();
    if ($stateStore) {
      const sign = event.code.includes("Right") ? 1 : -1;
      const range = $stateStore.getViewRange();
      const delta = (sign * (range[1] - range[0])) / 4;
      stateStore.update((s) => {
        s.setViewRange([range[0] + delta, range[1] + delta]);
        return s;
      });
    }
  }

  async function onVerticalArrow(event: KeyboardEvent) {
    event.preventDefault();
    if ($stateStore && canvasHeight < scrollHeight) {
      const sign = event.code.includes("Down") ? 1 : -1;
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
    panState = undefined;
    internalScrollTop(detail.yRatio * scrollHeight);
    stateStore.update((s) => {
      s.setViewRange([detail.xBegin, detail.xEnd]);
      return s;
    });
    await stateManager.fetchDynData();
  }

  function internalScrollTop(value: number) {
    div.scrollTo({ top: value });
  }
</script>

<svelte:window on:keydown={handleKeydown} bind:innerWidth={windowInnerWidth} />

{#if stateStore}
  <Loader loading={!$stateStore.ready} error={initializationError}>
    <div slot="body" class="flex flex-col">
      <div class="main">
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
            {#each $stateStore.processes as p (p.processId)}
              <div>
                <TimelineProcess
                  process={p}
                  {stateStore}
                  rootStartTime={stateManager.rootStartTime}
                  on:zoom={(e) => onZoom(e.detail)}
                />
              </div>
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
        <span class="range">
          <TimelineRange {stateStore} />
        </span>
      </div>
    </div>
  </Loader>
{/if}

<style lang="postcss">
  .main {
    overflow-x: hidden;
    overflow-y: hidden;
    position: relative;
  }

  .canvas {
    max-height: calc(100vh - 150px);
    background-color: #fcfcfc;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    @apply gap-y-1;
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
