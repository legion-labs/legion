<div class="root">
  <div class="tabs">
    <div class="header">
      <div class="header-container">
        <slot name="header" />
      </div>
    </div>
    <div class="tabs-filler-bg">
      <div class="tabs-filler" />
    </div>
  </div>
  <div class="content">
    <slot name="content" />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col w-full h-full;
  }

  .tabs {
    @apply flex flex-row h-7;
  }

  .header {
    @apply flex bg-black flex-shrink-0 rounded-tl-lg;
  }

  .header-container {
    @apply flex flex-row items-center bg-gray-700 px-2 rounded-t-lg text-sm;
  }

  .tabs-filler-bg {
    @apply flex bg-gray-700 w-full rounded-tr-lg;
  }

  .tabs-filler {
    @apply bg-black rounded-tr-lg rounded-bl-lg w-full;
  }

  .content {
    @apply bg-gray-700 overflow-auto h-full w-full rounded-b-lg;
  }
</style>
