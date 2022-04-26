<script lang="ts">
  import { spanPixelHeight as pixel } from "./Values/TimelineValues";

  export let threadTitle: string = "";
  export let threadName: string;
  export let processCollapsed: boolean;
  export let maxDepth: number;

  let collapsed = false;

  $: pixelHeight = processCollapsed ? 0 : Math.max(pixel, maxDepth * pixel);
  $: style = collapsed
    ? `max-height:${processCollapsed ? 0 : pixel}px`
    : `height:${pixelHeight}px`;

  export function setCollapse(state: boolean) {
    collapsed = state;
  }
</script>

<div class="flex items-start overflow-hidden" {style}>
  <div
    class="thread text-sm bg-background text-content-60 px-1 overflow-hidden cursor-pointer mr-1 self-stretch"
    on:click={() => setCollapse(!collapsed)}
    title={threadTitle}
  >
    <span class="text">
      <i class={`pr-1 bi bi-${!collapsed ? "eye" : "eye-slash"}-fill`} />
      <span class="capitalize whitespace-nowrap overflow-hidden text-ellipsis"
        >{threadName}</span
      >
    </span>
    <slot name="details" />
  </div>
  <slot name="canvas" />
</div>

<style lang="postcss">
  div :global(.text) {
    @apply flex;
  }

  .thread {
    min-width: var(--thread-item-length);
  }
</style>
