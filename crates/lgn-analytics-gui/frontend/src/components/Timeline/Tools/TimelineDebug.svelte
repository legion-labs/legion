<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { getLodFromPixelSizeMs, MergeThresholdForLOD } from "@/lib/lod";
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  export let store: TimelineStateStore;
  let pixelSize: number;
  $: lod = getLodFromPixelSizeMs(pixelSize);
  $: mergeThreshold = MergeThresholdForLOD(lod);
  $: {
    const vr = $store.getViewRange();
    pixelSize = (vr[1] - vr[0]) / $store.canvasWidth;
  }

  function* getDebugEntries() {
    yield `Pixel size: ${formatExecutionTime(pixelSize)}`;
    yield `Lod: ${lod}`;
    yield `Threshold: ${formatExecutionTime(mergeThreshold)}`;
    yield `Events: ${$store.eventCount.toLocaleString()}`;
  }
</script>

<div title={Array.from(getDebugEntries()).join("\n")}>
  <i class="bi bi-question-circle-fill" />
</div>

<style lang="postcss">
  i {
    @apply text-slate-400;
  }
</style>
