<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount, tick } from "svelte";
  import type { Unsubscriber } from "svelte/store";

  import { debounced } from "@lgn/web-client/src/lib/store";

  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { DrawSelectedRange } from "@/lib/time_range_selection";

  import type { TimelineTrackCanvasBaseDrawer } from "./Drawing/TimelineTrackCanvasBaseDrawer";
  import { TimelineContext } from "./Stores/TimelineContext";
  import { spanPixelHeight } from "./Values/TimelineValues";

  export let stateStore: TimelineStateStore;
  export let processCollapsed: boolean;
  export let maxDepth: number;
  export let drawerBuilder: () => TimelineTrackCanvasBaseDrawer;
  export let dataObject: any;

  let canvas: HTMLCanvasElement | null;
  let ctx: CanvasRenderingContext2D;
  let height: number;
  let intersectionObserver: IntersectionObserver;
  let searchSubscription: Unsubscriber;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  const searchStore = debounced(TimelineContext.search, 100);
  const canvasDrawer = drawerBuilder();

  $: if (dataObject) {
    draw();
  }

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
      canvasDrawer.draw($searchStore);
      if ($stateStore.selectionState) {
        DrawSelectedRange(
          canvas,
          ctx,
          $stateStore.selectionState,
          $stateStore.getViewRange()
        );
      }
    }
  }
</script>

<div
  style={`width:${$stateStore.canvasWidth}px`}
  class="timeline-item"
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
