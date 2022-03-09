<script lang="ts">
  import { TimelineStateManager } from "@/lib/Timeline/TimelineStateManager";
  import { onMount } from "svelte";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { BarLoader } from "svelte-loading-spinners";
  import TimelineDetails from "./TimelineDetails.svelte";
  import {
    NewSelectionState,
    RangeSelectionOnMouseDown,
    RangeSelectionOnMouseMove,
  } from "@/lib/time_range_selection";
  import TimelineAction from "./TimelineAction.svelte";
  import TimelineDebug from "./TimelineDebug.svelte";
  import TimelineThreadElement from "./TimelineThreadItem.svelte";
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

  $: if (windowInnerWidth) {
    canvasWidth = windowInnerWidth - 220;
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
        $stateStore.selectionState,
        canvasWidth,
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
      if (event.target instanceof HTMLCanvasElement) {
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
          div.scrollBy(0, -event.movementY);
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
  }

  function onMouseDown(event: MouseEvent) {
    if (RangeSelectionOnMouseDown(event, $stateStore.selectionState)) {
      stateStore.update((s) => {
        s.currentSelection = s.selectionState.selectedRange;
        return s;
      });
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.code == "Escape" && $stateStore.currentSelection) {
      stateStore.update((s) => {
        s.currentSelection = undefined;
        s.selectionState = NewSelectionState();
        return s;
      });
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

<div {style}>
  <div
    bind:this={div}
    class="canvas "
    on:mousedown|preventDefault={(e) => onMouseDown(e)}
    on:mousemove|preventDefault={(e) => onMouseMove(e)}
  >
    {#if stateManager}
      {#each Object.entries($stateStore.threads) as [key, thread] (key)}
        <TimelineThreadElement
          {thread}
          {stateStore}
          width={canvasWidth}
          rootStartTime={stateManager.rootStartTime}
          on:zoom={(e) => onZoom(e.detail)}
        />
      {/each}
    {/if}
  </div>
</div>

{#if $stateStore?.ready && stateManager?.process}
  <div class="action-container">
    <TimelineAction
      {processId}
      process={stateManager.process}
      timeRange={$stateStore.currentSelection}
    />
  </div>
{/if}

<style lang="postcss">
  .canvas {
    max-height: calc(100vh - 150px);
    overflow-y: visible;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    @apply gap-y-1;
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
