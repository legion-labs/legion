<script lang="ts">
  import { getContext } from "svelte";
  import { createEventDispatcher, onMount } from "svelte";

  import { TimelineMinimapViewport } from "../Lib/TimelineViewport";
  import type { TimelineStateStore } from "../Stores/TimelineStateStore";

  export let stateStore: TimelineStateStore;
  export let canvasHeight: number;
  export let scrollHeight: number;
  export let scrollTop: number;

  const threadItemLength = getContext("thread-item-length");
  const canvasToMinimapRatio = 5;
  const minimapBreakpoint = 300;
  const bottomPadding = 20;
  const leftPadding = 4;

  const wheelDispatch = createEventDispatcher<{ zoom: WheelEvent }>();
  const minimapDispatch = createEventDispatcher<{
    tick: {
      xBegin: number;
      xEnd: number;
      yRatio: number;
    };
  }>();

  let width: number;
  let height: number;
  let visible: boolean;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let viewport = new TimelineMinimapViewport();
  let x: number;
  let y: number;

  $: canvasWidth = $stateStore?.canvasWidth;
  $: top = canvasHeight - height + bottomPadding;
  $: left = canvasWidth - width - leftPadding + threadItemLength;
  $: style = `top:${top}px;left:${left}px`;
  $: [x, y] = $stateStore?.viewRange ?? [-Infinity, Infinity];
  $: (x || y) && draw();

  $: if (canvasWidth || canvasHeight) {
    width = Math.ceil(canvasWidth / canvasToMinimapRatio);
    height = Math.ceil(canvasHeight / canvasToMinimapRatio);
    draw();
  }

  $: if (scrollHeight || scrollTop) {
    draw();
  }

  $: if ($stateStore?.viewRange) {
    visible = $stateStore.isFullyVisible();
  }

  onMount(() => {
    const context = canvas.getContext("2d");
    if (context) {
      ctx = context;
    }
  });

  async function draw() {
    if (visible && ctx && canvasHeight > minimapBreakpoint) {
      requestAnimationFrame(() => {
        if (canvas) {
          canvas.width = width;
          canvas.height = height;
          ctx.globalAlpha = 0.66;
          ctx.fillStyle = "black";
          ctx.fillRect(0, 0, width, height);
          drawViewport();
        }
      });
    }
  }

  function drawViewport() {
    if (ctx && visible) {
      ctx.save();
      ctx.globalAlpha = 0.5;
      ctx.fillStyle = "#fea446";
      updateViewport();
      const minPixelSize = 4;
      ctx.fillRect(
        Math.max(
          Math.min(viewport.x, width - minPixelSize),
          -viewport.width + minPixelSize
        ),
        viewport.y,
        Math.max(minPixelSize, viewport.width),
        Math.max(minPixelSize, viewport.height)
      );
      ctx.restore();
    }
  }

  function updateViewport() {
    const viewRange = $stateStore.viewRange;
    const maxViewRange = viewRange[1] - viewRange[0];
    const maxRange = $stateStore.getMaxRange();
    let x = viewRange[0] / maxRange;
    let y =
      canvasHeight / (isFinite(scrollHeight) ? scrollHeight : canvasHeight);
    const xViewportSize = (maxViewRange / maxRange) * width;
    const yViewportSize = y * height;
    let yLocation = scrollTop / (scrollHeight - canvasHeight);
    yLocation = !isFinite(yLocation) ? 1 : yLocation;
    viewport.set(
      x * width,
      yLocation * (height - yViewportSize),
      xViewportSize,
      yViewportSize
    );
  }

  async function onMouseEvent(mouseEvent: MouseEvent) {
    const beginRatio = (mouseEvent.offsetX - viewport.width / 2) / width;
    const endRatio = (mouseEvent.offsetX + viewport.width / 2) / width;
    const maxRange = $stateStore.getMaxRange();
    const xBegin = beginRatio * maxRange;
    const xEnd = endRatio * maxRange;
    const yRatio = mouseEvent.offsetY / height;
    minimapDispatch("tick", {
      xBegin,
      xEnd,
      yRatio,
    });
  }
</script>

<span style={visible ? "display:block" : "display:none"}>
  <canvas
    class="absolute"
    bind:this={canvas}
    on:mousemove|preventDefault={(e) => e.buttons === 1 && onMouseEvent(e)}
    on:mousedown|preventDefault={(e) => onMouseEvent(e)}
    on:wheel|preventDefault={(e) => wheelDispatch("zoom", e)}
    {style}
  />
</span>
