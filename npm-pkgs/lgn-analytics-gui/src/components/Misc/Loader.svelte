<script lang="ts">
  import { BarLoader } from "svelte-loading-spinners";

  export let loading: boolean;
  export let error = "";
</script>

{#if error}
  <div>
    <span class="text-sm placeholder">
      {error}
    </span>
  </div>
{:else if loading}
  <div>
    <BarLoader />
  </div>
{:else}
  <slot name="body" />
{/if}

<style lang="postcss">
  div {
    @apply flex items-center justify-center;
    height: 90vh;
  }
</style>
