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

<div class="flex items-start main" {style}>
  <div
    class="thread px-1"
    on:click={() => setCollapse(!collapsed)}
    title={threadTitle}
  >
    <span class="text">
      <i class={`icon bi bi-${!collapsed ? "eye" : "eye-slash"}-fill`} />
      <span class="thread-name">{threadName}</span>
    </span>
    <slot name="details" />
  </div>
  <slot name="canvas" />
</div>

<style lang="postcss">
  .main {
    overflow: hidden;
  }

  .thread-name {
    @apply capitalize;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  div :global(.text) {
    @apply flex;
  }

  .icon {
    @apply pr-1;
  }

  .thread {
    @apply text-sm text-slate-400;
    min-width: var(--thread-item-length);
    overflow: hidden;
    cursor: pointer;
    background-color: #f0f0f0;
    margin-right: 4px;
    align-self: stretch;
  }
</style>
