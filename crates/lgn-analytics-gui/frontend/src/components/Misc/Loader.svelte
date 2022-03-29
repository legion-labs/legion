<script lang="ts">
  import { BarLoader } from "svelte-loading-spinners";

  export let loading: boolean;
</script>

{#if loading}
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
