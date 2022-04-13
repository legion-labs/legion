<div class="root">
  <div class="tabs-container" />
  <div class="content">
    <slot />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col w-full h-full;
  }

  .tabs-container {
    @apply flex flex-shrink-0 h-8 bg-black rounded-tl-sm;
  }

  .content {
    @apply bg-gray-700 flex-1 w-full rounded-b-sm;
  }
</style>
