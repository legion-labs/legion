<script lang="ts">
  export let disabled = false;

  // eslint-disable-next-line @typescript-eslint/prefer-as-const
  export let variant: "success" = "success";

  export let title: string | null = null;
</script>

<div
  class="root"
  class:disabled
  class:success={variant === "success" && !disabled}
  {title}
>
  {#if !disabled}
    <div class="ping" />
  {/if}
  <div class="point" />
</div>

<style lang="postcss">
  .root {
    @apply relative flex h-full w-full;
  }

  .ping {
    @apply animate-ping absolute inline-flex h-full w-full rounded-full opacity-75;
  }

  .point {
    @apply relative inline-flex rounded-full h-full w-full;
  }

  .disabled .point {
    @apply bg-gray-500;
  }

  .success .ping {
    @apply bg-green-600;
  }

  .success .point {
    @apply bg-green-700;
  }
</style>
