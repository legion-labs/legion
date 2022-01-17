<script lang="ts">
  import clickOutside from "../../actions/clickOutside";

  /** The `string` value is just an identifier that can be used to display any kind of tab label */
  export let tabs: string[] = [];

  /** Optionally change the value used as key during the iteration */
  export let key: (tab: string, index: number) => string = (tab) => tab;

  export let activeTab = tabs[0];

  let isFocused = false;
</script>

<div
  class="root"
  on:click={() => (isFocused = true)}
  use:clickOutside={() => (isFocused = false)}
>
  <div class="header">
    <div class="tabs">
      {#each tabs as tab, index (key(tab, index))}
        <div
          class="tab"
          class:tab-inactive={activeTab !== tab}
          class:tab-active={activeTab === tab}
          on:click={() => (activeTab = tab)}
        >
          <slot name="tab" {tab} {isFocused} />
        </div>
      {/each}
    </div>
    <div
      class="tabs-filler-bg"
      class:last-tabs-filler-bg={activeTab === tabs[tabs.length - 1]}
    >
      <div class="tabs-filler" />
    </div>
  </div>

  <div class="content">
    <slot name="content" {isFocused} {activeTab} />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col w-full h-full;
  }

  .header {
    /* TODO: Instead of hiding the overflow we should display it properly */
    @apply flex flex-row flex-shrink-0 h-8 overflow-hidden;
  }

  .tabs {
    @apply flex bg-black flex-shrink-0 rounded-tl-sm;
  }

  .tab {
    @apply flex flex-row relative items-center bg-gray-700 px-2 cursor-pointer first:rounded-tl-sm last:rounded-tr-sm;
  }

  .tab-inactive {
    @apply relative bg-gray-500 text-white;
  }

  .tab-active {
    @apply relative z-10 border-b-2 border-orange-700 shadow-lg;
  }

  .tabs-filler-bg {
    @apply flex bg-gray-400 w-full rounded-tr-sm;
  }

  .last-tabs-filler-bg {
    @apply bg-gray-700;
  }

  .tabs-filler {
    @apply bg-black rounded-tr-sm w-full;
  }

  .content {
    @apply bg-gray-700 flex-1 w-full rounded-b-sm overflow-hidden;
  }
</style>
