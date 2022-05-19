<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";

  import type { TimelineStateStore } from "../Stores/TimelineStateStore";

  export let stateStore: TimelineStateStore;

  let percent: number = 0;

  $: if ($stateStore?.currentSelection) {
    const width = $stateStore?.canvasWidth;
    const [begin, end] = $stateStore.viewRange;
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
  <div class="flex flex-row items-center w-full h-4 overflow-hidden">
    <div class="min-w-thread-item" />
    <div class="overflow-hidden" style={`width:${$stateStore?.canvasWidth}px`}>
      <div
        class="h-full flex flex-row text-xs placeholder"
        style={`transform: translate(${percent}%);`}
      >
        <div class="flex flex-row h-full -translate-x-1/2">
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
