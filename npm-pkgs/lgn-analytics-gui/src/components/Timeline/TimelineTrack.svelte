<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
  import type { Unsubscriber } from "svelte/store";

  import { debounced } from "@lgn/web-client/src/lib/store";

  import type { TimelineTrackCanvasBaseDrawer } from "./Drawing/TimelineTrackCanvasBaseDrawer";
  import type { TimelineStateStore } from "./Lib/TimelineStateStore";
  import { TimelineContext } from "./Stores/TimelineContext";
  import { spanPixelHeight } from "./Values/TimelineValues";

  export let stateStore: TimelineStateStore;
  export let processCollapsed: boolean;
  export let maxDepth: number;
  export let drawerBuilder: () => TimelineTrackCanvasBaseDrawer;

  let canvas: HTMLCanvasElement | null;
  let ctx: CanvasRenderingContext2D;
  let height: number;
  let intersectionObserver: IntersectionObserver;
  let searchSubscription: Unsubscriber;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  const searchStore = debounced(TimelineContext.search, 100);
  const canvasDrawer = drawerBuilder();

  $: height = Math.max(spanPixelHeight, maxDepth * spanPixelHeight);

  $: if (
    $stateStore?.scopes ||
    $stateStore?.getViewRange() ||
    $stateStore?.canvasWidth ||
    $stateStore?.currentSelection
  ) {
    draw();
  }

  onMount(() => {
    if (canvas) {
      const observer = new IntersectionObserver(onIntersection, {
        threshold: [0, 1],
      });
      observer.observe(canvas);
      const context = canvas.getContext("2d");
      if (context) {
        canvasDrawer.initialize(canvas, context);
        ctx = context;
      }
    }
    searchSubscription = searchStore.subscribe((_) => {
      draw();
    });
  });

  onDestroy(() => {
    if (intersectionObserver) {
      intersectionObserver.disconnect();
    }
    if (searchSubscription) {
      searchSubscription();
    }
  });

  async function onIntersection(entries: IntersectionObserverEntry[]) {
    const visible = entries[0].intersectionRatio > 0;
    if (visible) {
      await draw();
    }
  }

  async function draw() {
    if (canvas && ctx && !processCollapsed) {
      await tick();
      canvasDrawer.draw($searchStore, $stateStore);
      drawSelectedRange();
    }
  }

  function drawSelectedRange() {
    if (!canvas) {
      return;
    }

    if (!$stateStore.currentSelection) {
      return;
    }

    const selectionState = $stateStore.currentSelection;
    const viewRange = $stateStore.getViewRange();
    const [begin, end] = viewRange;
    const invTimeSpan = 1.0 / (end - begin);
    const canvasWidth = canvas.clientWidth;
    const canvasHeight = canvas.clientHeight;
    const msToPixelsFactor = invTimeSpan * canvasWidth;
    const [beginSelection, endSelection] = selectionState;
    const beginPixels = (beginSelection - begin) * msToPixelsFactor;
    const endPixels = (endSelection - begin) * msToPixelsFactor;

    ctx.fillStyle = "rgba(140, 140, 140, 0.3)";
    ctx.fillRect(beginPixels, 0, endPixels - beginPixels, canvasHeight);
  }
</script>

<div
  style={`width:${$stateStore.canvasWidth}px`}
  on:wheel|preventDefault={(e) => wheelDispatch("zoom", e)}
>
  <canvas width={$stateStore.canvasWidth} {height} bind:this={canvas} />
</div>

<style>
  div {
    align-self: stretch;
    background-color: #f0f0f0;
    cursor: pointer;
  }
</style>
