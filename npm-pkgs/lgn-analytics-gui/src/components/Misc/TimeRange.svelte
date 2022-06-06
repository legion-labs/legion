<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";

  export let width: number;

  export let selectionRange: [number, number];

  export let viewRange: [number, number];

  let percent: number = 0;

  $: {
    const [begin, end] = viewRange;
    const invTimeSpan = 1.0 / (end - begin);
    const msToPixelsFactor = invTimeSpan * width;
    const [beginSelection, endSelection] = selectionRange;
    const beginPixel = Math.max(0, (beginSelection - begin) * msToPixelsFactor);
    const endPixel = Math.min(width, (endSelection - begin) * msToPixelsFactor);
    const centerPixel = (beginPixel + endPixel) / 2;

    percent = (100 * centerPixel) / width;
  }
</script>

<div class="h-4 overflow-hidden" style={`width:${width}px`}>
  <div
    class="h-full flex flex-row text-xs placeholder"
    style={`transform: translate(${percent}%);`}
  >
    <div class="flex flex-row h-full -translate-x-1/2">
      <i class="bi bi-arrow-left-short" />
      {formatExecutionTime(selectionRange[1] - selectionRange[0])}
      <i class="bi bi-arrow-right-short" />
    </div>
  </div>
</div>
