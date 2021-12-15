<script lang="ts">
  import clickOutside from "@/actions/clickOutside";

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
    @apply flex flex-row flex-shrink-0 h-8;
  }

  .tabs {
    @apply flex bg-black flex-shrink-0 rounded-tl-lg;
  }

  .tab {
    @apply flex flex-row relative items-center bg-gray-700 px-3 rounded-t-lg cursor-pointer;
  }

  .tab-inactive {
    @apply relative bg-gray-400;
  }

  .tab-inactive:last-child:after {
    content: "";
    box-shadow: calc(0px - theme("spacing.3")) 0px 0px 0px
      theme("colors.gray.400");
    @apply absolute -right-6 bottom-0 w-6 h-3 rounded-bl-full;
  }

  .tab-active {
    @apply relative z-10;
  }

  .tab-active:not(:first-child):before {
    content: "";
    box-shadow: theme("spacing.3") 0px 0px 0px theme("colors.gray.700");
    @apply absolute -left-6 bottom-0 w-6 h-3 rounded-br-full;
  }

  .tab-active:after {
    content: "";
    box-shadow: calc(0px - theme("spacing.3")) 0px 0px 0px
      theme("colors.gray.700");
    @apply absolute -right-6 bottom-0 w-6 h-3 rounded-bl-full;
  }

  .tabs-filler-bg {
    @apply flex bg-gray-400 w-full rounded-tr-lg;
  }

  .last-tabs-filler-bg {
    @apply bg-gray-700;
  }

  .tabs-filler {
    @apply bg-black rounded-tr-lg w-full;
  }

  .content {
    @apply bg-gray-700 flex-1 w-full rounded-b-lg overflow-hidden;
  }
</style>
