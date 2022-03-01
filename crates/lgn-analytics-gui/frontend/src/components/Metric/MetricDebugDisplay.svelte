<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { MetricState } from "@/lib/Metric/MetricState";
  import { MetricStreamer } from "@/lib/Metric/MetricStreamer";
  import { Writable } from "svelte/store";
  import TimeRangeDetails from "../TimeRangeDetails.svelte";
  export let width: number;
  export let mainWidth: number;
  export let updateTime: number;
  export let transform: d3.ZoomTransform;
  export let lod: number;
  export let pixelSizeNs: number;
  export let deltaMs: number;
  export let metricStreamer: MetricStreamer;
  export let totalMinMs: number;
  export let currentMinMs: number;
  export let totalMaxMs: number;
  export let currentMaxMs: number;
  export let brushStart: number;
  export let brushEnd: number;
  export let metricStore: Writable<MetricState[]>;
  export let id: string;
</script>

<div class="grid grid-cols-3">
  <div style="font-size:0.8rem">
    <div><span class="font-bold">Width</span>: {width}</div>
    <div><span class="font-bold"> Main Width</span>: {mainWidth}</div>
    <br />
    <div>
      <span class="font-bold">Update Time</span>: {updateTime} ms
    </div>
    <div>
      <span class="font-bold">Transform</span>
      <span class="font-bold">X</span>
      {transform.x.toFixed(2)}
      <span class="font-bold">Y</span>
      {transform.y.toFixed(2)}
    </div>
    <ul>
      <li>
        <span class="font-bold">Zoom</span>
        {transform.k}
      </li>
      <li>
        <span class="font-bold">Lod</span>
        {lod}
      </li>
      <li>
        <span class="font-bold">Pixel size</span>
        {formatExecutionTime(pixelSizeNs / 1_000_000)}
      </li>
      <li>
        <span class="font-bold">Delta Ms</span>
        {formatExecutionTime(deltaMs)}
      </li>
      <br />
      <li>
        <span class="font-bold">Min</span>
        {totalMinMs.toFixed(2)}
      </li>
      <li>
        <span class="font-bold">Current Min</span>
        {currentMinMs.toFixed(2)}
      </li>
      <li>
        <span class="font-bold">Max</span>
        {totalMaxMs.toFixed(2)}
      </li>
      <li>
        <span class="font-bold">Current Max</span>
        {currentMaxMs.toFixed(2)}
      </li>
      <li>
        <span class="font-bold">BrushStart</span>
        {brushStart}
        /
        <span class="font-bold">BrushEnd</span>
        {brushEnd}
      </li>
    </ul>
  </div>
  <div style="font-size:0.8rem">
    {#if metricStreamer}
      <ul>
        {#each $metricStore as ms}
          {#if ms.canBeDisplayed()}
            <li>
              {ms.name} (unit: {ms.unit})<br />
              {ms.min} _ {ms.max} ({formatExecutionTime(ms.max - ms.min)})<br />
              {#each Array.from(ms.getViewportBlocks(currentMinMs, currentMaxMs)) as b}
                <div style="font-size:0.7rem">
                  {b.blockId}
                  {b.minMs.toFixed(0)}
                  {b.maxMs.toFixed(0)} ({formatExecutionTime(
                    b.maxMs - b.minMs
                  )}) ({Array.from(
                    b.getPoints(currentMinMs, currentMaxMs, lod, true)
                  ).length})
                </div>
              {/each}
            </li>
          {/if}
        {/each}
      </ul>
    {/if}
  </div>
  <div>
    <TimeRangeDetails timeRange={[brushStart, brushEnd]} processId={id} />
  </div>
</div>
