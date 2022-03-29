<script lang="ts">
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { formatExecutionTime } from "@/lib/format";

  export let stateStore: TimelineStateStore;
  let percent: number;

  $: if (
    $stateStore?.currentSelection &&
    $stateStore.selectionState.selectedRange
  ) {
    const width = $stateStore?.canvasWidth;
    const [begin, end] = $stateStore.getViewRange();
    const invTimeSpan = 1.0 / (end - begin);
    const msToPixelsFactor = invTimeSpan * width;
    const [beginSelection, endSelection] =
      $stateStore.selectionState.selectedRange;
    const beginPixel = (beginSelection - begin) * msToPixelsFactor;
    const endPixel = (endSelection - begin) * msToPixelsFactor;
    const centerPixel = (beginPixel + endPixel) / 2;
    percent = (100 * centerPixel) / width;
  }
</script>

{#if $stateStore?.currentSelection}
  <div class="flex flex-row">
    <div class="block" />
    <div class="parent" style={`width:${$stateStore?.canvasWidth}px`}>
      <div class="range" style={`left:${percent}%`}>
        <div class="child">
          <i class="bi bi-arrow-left-short" />
          {formatExecutionTime(
            $stateStore.currentSelection[1] - $stateStore.currentSelection[0]
          )}
          <i class="bi bi-arrow-right-short" />
        </div>
      </div>
    </div>
  </div>
{/if}

<style lang="postcss">
  .block {
    @apply bg-slate-50;
    z-index: 1;
    min-width: calc(var(--thread-item-length) + 6px);
  }

  .range {
    @apply text-xs text-slate-400;
    position: relative;
  }

  .child {
    transform: translateX(-50%);
  }
</style>
