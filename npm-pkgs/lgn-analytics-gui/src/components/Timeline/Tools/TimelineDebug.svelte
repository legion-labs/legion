<script lang="ts">
  import { getContext } from "svelte";

  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";

  import { formatExecutionTime } from "@/lib/format";
  import { MergeThresholdForLOD, getLodFromPixelSizeMs } from "@/lib/lod";

  import type { TimelineStateStore } from "../Stores/TimelineStateStore";

  export let store: TimelineStateStore;

  const { t } = getContext(l10nOrchestratorContextKey);
  let pixelSize: number;
  let lod: number;
  let mergeThreshold: number;
  let title: string;

  $: {
    const [begin, end] = $store.viewRange;

    pixelSize = (end - begin) / $store.canvasWidth;
    lod = getLodFromPixelSizeMs(pixelSize);
    mergeThreshold = MergeThresholdForLOD(lod);

    title = $t("timeline-debug-tooltip", {
      events: $store.eventCount.toLocaleString(),
      lod,
      pixelSize: formatExecutionTime(pixelSize),
      threshold: formatExecutionTime(mergeThreshold),
    });
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
