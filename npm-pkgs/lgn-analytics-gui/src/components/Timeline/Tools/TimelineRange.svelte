<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";

  import type { TimelineStateStore } from "../Stores/TimelineStateStore";

  export let stateStore: TimelineStateStore;
  let percent: number;

  $: if ($stateStore?.currentSelection) {
    const width = $stateStore?.canvasWidth;
    const [begin, end] = $stateStore.getViewRange();
    const invTimeSpan = 1.0 / (end - begin);
    const msToPixelsFactor = invTimeSpan * width;
    const [beginSelection, endSelection] = $stateStore.currentSelection;
    const beginPixel = (beginSelection - begin) * msToPixelsFactor;
    const endPixel = (endSelection - begin) * msToPixelsFactor;
    const centerPixel = (beginPixel + endPixel) / 2;
    percent = (100 * centerPixel) / width;
  }
</script>

{#if $stateStore?.currentSelection}
  <div class="flex flex-row">
    <div class="block" />
    <div class="overflow-hidden" style={`width:${$stateStore?.canvasWidth}px`}>
      <div
        class="flex text-xs text-placeholder relative "
        style={`left:${percent}%`}
      >
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
    background-color: transparent;
    z-index: 1;
    min-width: calc(var(--thread-item-length) + 6px);
  }

  .child {
    transform: translateX(-50%);
  }
</style>
