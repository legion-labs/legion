<script lang="ts">
  import { afterUpdate, getContext, onMount, tick } from "svelte";
  import { useLocation } from "svelte-navigator";
  import { get } from "svelte/store";

  import { threadItemLengthContextKey } from "@/constants";
  import { loadingStore } from "@/lib/Misc/LoadingStore";
  import { endQueryParam, startQueryParam } from "@/lib/time";

  import CallGraph from "../CallGraphHierachy/CallGraphHierachy.svelte";
  import Loader from "../Misc/Loader.svelte";
  import { TimelineStateManager } from "./Stores/TimelineStateManager";
  import type { TimelineStateStore } from "./Stores/TimelineStateStore";
  import TimelineProcess from "./TimelineProcess.svelte";
  import TimelineAction from "./Tools/TimelineAction.svelte";
  import TimelineAxis from "./Tools/TimelineAxis.svelte";
  import TimelineMinimap from "./Tools/TimelineMinimap.svelte";
  import TimelineRange from "./Tools/TimelineRange.svelte";
  import TimelineSearch from "./Tools/TimelineSearch.svelte";
  import { pixelMargin } from "./Values/TimelineValues";

  export let processId: string;

  const location = useLocation();

  const threadItemLength = getContext<number>(threadItemLengthContextKey);

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
    const url = new URLSearchParams($location.search);
    const s = url.get(startQueryParam);
    const start = s != null ? Number.parseFloat(s) : null;
    const e = url.get(endQueryParam);
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

{#if stateStore}
  <Loader loading={!$stateStore.ready} error={initializationError}>
    <div slot="body" class="flex flex-col">
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
      <div class="main relative flex flex-col">
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
        <div class="mt-3">
          <TimelineRange {stateStore} />
        </div>
        {#if callGraphBegin && callGraphEnd}
          <div class="mt-3 basis-1/5">
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
  </Loader>
{/if}

<style lang="postcss">
  .main {
    height: calc(100vh - 130px);
  }

  .canvas {
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    @apply gap-y-1;
  }

  ::-webkit-scrollbar {
    width: 20px;
  }

  ::-webkit-scrollbar-track {
    background-color: transparent;
  }

  ::-webkit-scrollbar-corner {
    background: rgba(0, 0, 0, 0);
  }

  ::-webkit-scrollbar-thumb {
    background-color: #454545;
    border-radius: 20px;
    border: 6px solid transparent;
    background-clip: content-box;
  }

  ::-webkit-scrollbar-thumb:hover {
    background-color: #707070;
  }
</style>
