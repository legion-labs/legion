<script lang="ts">
  import type { TimelineStateStore } from "@/lib/Timeline/TimelineStateStore";
  import { formatExecutionTime } from "@/lib/format";
  import { MergeThresholdForLOD, getLodFromPixelSizeMs } from "@/lib/lod";

  export let store: TimelineStateStore;

  let pixelSize: number;
  let lod: number;
  let mergeThreshold: number;
  let title: string;

  $: {
    const vr = $store.getViewRange();
    pixelSize = (vr[1] - vr[0]) / $store.canvasWidth;
    lod = getLodFromPixelSizeMs(pixelSize);
    mergeThreshold = MergeThresholdForLOD(lod);
    title = Array.from(getDebugEntries()).join("\n");
  }

  function* getDebugEntries() {
    yield `Pixel size: ${formatExecutionTime(pixelSize)}`;
    yield `Lod: ${lod}`;
    yield `Threshold: ${formatExecutionTime(mergeThreshold)}`;
    yield `Events: ${$store.eventCount.toLocaleString()}`;
  }
</script>

<div {title}>
  <i class="bi bi-question-circle-fill" />
</div>

<style lang="postcss">
  i {
    @apply text-slate-400;
  }
</style>
