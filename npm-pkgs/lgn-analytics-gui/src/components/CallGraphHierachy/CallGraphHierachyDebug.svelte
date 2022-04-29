<script lang="ts">
  import type { CumulatedCallGraphStore } from "@/lib/CallGraph/CallGraphStore";

  export let begin: number;
  export let end: number;
  export let store: CumulatedCallGraphStore;
</script>

<div class="text-xs">
  <div>
    Begin: {begin}
  </div>
  <div>
    End: {end}
  </div>
  <div class="text-headline">
    <div class="font-semibold">Scopes</div>
    {#each Object.values($store.scopes) as scope (scope.hash)}
      <div>
        {scope.name} ({scope.hash})
      </div>
    {/each}
    <div class="font-semibold">Cache</div>
    {#each Array.from($store.cache) as [k, v] (k)}
      <div>
        {v.streamName} ({v.streamHash})
      </div>
    {/each}
  </div>
</div>
