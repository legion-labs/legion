<script lang="ts">
  export let error: string;
</script>

<div class="error">
  <span class="text-sm placeholder">
    {error}
  </span>
</div>

<style lang="postcss">
  .error {
    @apply flex items-center justify-center h-[calc(100vh-3.5rem)];
  }
</style>
