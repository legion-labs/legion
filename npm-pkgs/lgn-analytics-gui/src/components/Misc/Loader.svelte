<script lang="ts">
  import { BarLoader } from "svelte-loading-spinners";
</script>

<div class="loader">
  <BarLoader />
</div>

<style lang="postcss">
  .loader {
    @apply flex items-center justify-center h-[calc(100vh-6rem)];
  }
</style>
