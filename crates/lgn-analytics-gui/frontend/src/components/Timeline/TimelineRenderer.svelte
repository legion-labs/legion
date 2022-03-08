<script lang="ts">
  import { TimelineStateManager } from "@/lib/Timeline/TimelineStateManager";
  import { onMount } from "svelte";
  import TimelineThread from "./TimelineThread.svelte";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { BarLoader } from "svelte-loading-spinners";
  import TimelineDetails from "./TimelineDetails.svelte";
  import {
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
    SelectionState,
  } from "@/lib/time_range_selection";
  import TimelineAction from "./TimelineAction.svelte";
  import TimelineDebug from "./TimelineDebug.svelte";
  export let processId: string;

  type PanState = {
    beginMouseX: number;
    beginMouseY: number;
    viewRange: [number, number];
  };

  let stateManager: TimelineStateManager;
  let windowInnerWidth: number;
  let stateStore: TimelineStateStore;
  let panState: PanState | undefined = undefined;
  let canvasWidth = NaN;
  let div: HTMLElement;
  let selectionState: SelectionState = NewSelectionState();
  let currentSelection: [number, number] | undefined;

  $: if (windowInnerWidth) {
    canvasWidth = windowInnerWidth - 80;
  }

  $: style = `display:${$stateStore?.ready ? "block" : "none"}`;

  onMount(async () => {
    loadingStore.reset();
    stateManager = new TimelineStateManager(processId);
    stateStore = stateManager.state;
    await stateManager.initAsync(windowInnerWidth);
  });

  async function onZoom(event: WheelEvent) {
    stateStore.update((s) => {
      s.setViewRangeFromWheel(s.getViewRange(), canvasWidth, event);
      return s;
    });

    await stateManager.fetchLodsAsync(windowInnerWidth);
  }

  function onMouseMove(event: MouseEvent) {
    if (
      RangeSelectionOnMouseMove(
        event,
        selectionState,
        canvasWidth,
        $stateStore.getViewRange()
      )
    ) {
      if (currentSelection != selectionState.selectedRange) {
        currentSelection = selectionState.selectedRange;
      }
      return;
    }

    if (event.buttons !== 1) {
      panState = undefined;
    } else if (!event.shiftKey) {
      if (!panState) {
        panState = {
          beginMouseX: event.offsetX,
          beginMouseY: event.offsetY,
          viewRange: stateStore.value.getViewRange(),
        };
      }

      const factor =
        (panState.viewRange[1] - panState.viewRange[0]) / canvasWidth;
      const offsetMs = factor * (panState.beginMouseX - event.offsetX);

      if (event.movementY) {
        div.scrollBy(0, -event.movementY / 2);
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
    }
  }

  function onMouseDown(event: MouseEvent) {
    if (RangeSelectionOnMouseDown(event, selectionState)) {
      currentSelection = selectionState.selectedRange;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.code == "Escape" && currentSelection) {
      currentSelection = undefined;
      selectionState = NewSelectionState();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} bind:innerWidth={windowInnerWidth} />

{#if stateStore && !$stateStore.ready}
  <div class="flex items-center justify-center loader">
    <BarLoader />
  </div>
{/if}

{#if stateManager?.process && $stateStore.ready}
  <div class="flex flex-row justify-between pb-2 pr-6">
    <TimelineDetails process={stateManager?.process} />
    <TimelineDebug {canvasWidth} store={stateStore} />
  </div>
{/if}

<div
  bind:this={div}
  class="canvas "
  {style}
  on:mousedown|preventDefault={(e) => onMouseDown(e)}
  on:mousemove|preventDefault={(e) => onMouseMove(e)}
>
  {#if stateManager}
    {#each Object.entries($stateStore.threads) as [key, thread] (key)}
      <TimelineThread
        {thread}
        {stateStore}
        {selectionState}
        {currentSelection}
        width={canvasWidth}
        range={$stateStore.getViewRange()}
        blocks={$stateStore.blocks}
        scopes={$stateStore.scopes}
        rootStartTime={stateManager.rootStartTime}
        on:zoom={(e) => onZoom(e.detail)}
      />
    {/each}
  {/if}
</div>

{#if $stateStore?.ready && stateManager?.process}
  <div class="action-container">
    <TimelineAction
      {processId}
      process={stateManager.process}
      timeRange={currentSelection}
    />
  </div>
{/if}

<style>
  .canvas {
    max-height: calc(100vh - 150px);
    overflow-y: visible;
    overflow-x: hidden;
  }

  .action-container {
    padding-top: 8px;
  }

  .loader {
    height: 90vh;
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
