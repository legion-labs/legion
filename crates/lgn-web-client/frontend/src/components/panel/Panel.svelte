<script lang="ts">
  import clickOutside from "../../actions/clickOutside";

  type Tab = $$Generic;

  /** The `string` value is just an identifier that can be used to display any kind of tab label */
  export let tabs: Tab[] = [];

  /** Optionally change the value used as key during the iteration */
  export let key: (tab: Tab, index: number) => Tab = (tab) => tab;

  export let activeTab: Tab | null | undefined = tabs[0];

  let isFocused = false;

  function focus() {
    isFocused = true;
  }

  function blur() {
    isFocused = false;
  }
</script>

<div class="root">
  <div class="tabs-container">
    <div class="tabs">
      {#each tabs as tab, index (key(tab, index))}
        <div
          class="tab"
          class:tab-inactive={activeTab !== tab}
          class:tab-active={activeTab === tab}
          on:click={() => (activeTab = tab)}
        >
          <slot name="tab" {tab} {isFocused} {activeTab} />
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

  {#if $$slots.header}
    <div class="header"><slot name="header" /></div>
  {/if}

  <div
    class="content"
    on:mousedown={focus}
    on:click-outside={blur}
    use:clickOutside
  >
    <slot name="content" {isFocused} {activeTab} />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col w-full h-full;
  }

  .tabs-container {
    /* TODO: Instead of hiding the overflow we should display it properly */
    @apply flex flex-row flex-shrink-0 h-8 overflow-hidden;
  }

  .tabs {
    @apply flex bg-black flex-shrink-0 rounded-tl-sm;
  }

  .tab {
    @apply flex flex-row relative items-center bg-gray-700 px-2 cursor-pointer first:rounded-tl-sm last:rounded-tr-sm border-b-2 border-transparent;
  }

  .tab-inactive {
    @apply relative bg-gray-500 bg-opacity-20 hover:bg-gray-700 hover:border-orange-700 text-white transition-colors;
  }

  .tab-active {
    @apply relative z-10 border-orange-700;
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

  .header {
    @apply w-full bg-gray-500 bg-opacity-50 border-b border-gray-800;
  }

  .content {
    @apply bg-gray-700 flex-1 w-full rounded-b-sm overflow-hidden;
  }
</style>
