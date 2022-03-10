<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { getLodFromPixelSizeMs, MergeThresholdForLOD } from "@/lib/lod";
  import { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  export let store: TimelineStateStore;
  export let canvasWidth: number;
  let pixelSize: number;
  $: lod = getLodFromPixelSizeMs(pixelSize);
  $: mergeThreshold = MergeThresholdForLOD(lod);
  $: {
    const vr = $store.getViewRange();
    pixelSize = (vr[1] - vr[0]) / canvasWidth;
  }
</script>

<div class="flex gap-2 select-none">
  <div>
    <span class="label">Pixel Size</span>
    <span class="value">{formatExecutionTime(pixelSize)}</span>
  </div>
  <div>
    <span class="label">Lod</span>
    <span class="value">{lod}</span>
  </div>
  <div>
    <span class="label">Threshold</span>
    <span class="value">{formatExecutionTime(mergeThreshold)}</span>
  </div>
  <div>
    <span class="label">Events</span>
    <span class="value">{$store.eventCount.toLocaleString()}</span>
  </div>
</div>

<style lang="postcss">
  .value {
    @apply text-sm  text-gray-400;
  }

  .label {
    @apply text-sm font-semibold text-gray-500;
  }
</style>
