<script lang="ts">
  import { formatExecutionTime } from "@/lib/format";
  import { endQueryParam, startQueryParam } from "@/lib/time";

  import L10n from "./L10n.svelte";

  export let timeRange: [number, number] | undefined;
  export let processId: string;

  let brushStart: number;
  let brushEnd: number;

  $: if (timeRange) {
    [brushStart, brushEnd] = timeRange;
  }
</script>

{#if !isNaN(brushStart) && !isNaN(brushEnd)}
  <div class="selected-time-range">
    <div>
      <div class="nav-link">
        <a
          href={`/cumulative-call-graph?process=${processId}&${startQueryParam}=${brushStart}&${endQueryParam}=${brushEnd}`}
        >
          <L10n id="metrics-open-cumulative-call-graph" />
        </a>
      </div>
      <div class="nav-link">
        <a
          href={`/timeline/${processId}?${startQueryParam}=${brushStart}&${endQueryParam}=${brushEnd}`}
        >
          <L10n id="metrics-open-timeline" />
        </a>
      </div>
    </div>
    <!-- TODO: Display the following the same way as in the timeline -->
    <div class="text-sm text-right">
      <div>
        <L10n id="metrics-selected-time-range" />
      </div>
      <div>
        <span class="font-bold"
          ><L10n id="metrics-selected-time-range-duration" />
        </span>
        <span>{formatExecutionTime(brushEnd - brushStart)}<span /></span>
      </div>
      <div>
        <span class="font-bold"
          ><L10n id="metrics-selected-time-range-beginning" />
        </span>
        <span>{formatExecutionTime(brushStart)}<span /></span>
      </div>
      <div>
        <span class="font-bold"
          ><L10n id="metrics-selected-time-range-end" />
        </span>
        <span>{formatExecutionTime(brushEnd)}<span /></span>
      </div>
    </div>
  </div>
{/if}

<style lang="postcss">
  .selected-time-range {
    @apply flex w-full justify-between;
  }

  .nav-link {
    @apply text-[#ca2f0f] underline;
  }
</style>
