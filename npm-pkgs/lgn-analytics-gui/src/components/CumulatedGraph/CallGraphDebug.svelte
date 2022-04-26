<script lang="ts">
  import type { CumulatedCallGraphStore } from "@/components/CumulatedGraph/Lib/CallGraphStore";

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
  <div class="text-content-87">
    <div class="font-semibold">Scopes</div>
    {#each Object.values($store.scopes) as scope (scope.hash)}
      <div>
        {scope.name} ({scope.hash})
      </div>
    {/each}
    <div class="font-semibold">Blocks</div>
    {#each Object.values($store.cache) as block (block.streamHash)}
      <div>
        {block.streamName} ({block.streamHash})
      </div>
    {/each}
  </div>
</div>
