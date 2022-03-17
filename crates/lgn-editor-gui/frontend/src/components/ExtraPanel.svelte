<!--
@Component
Contains all the "extra" panels like Logs or Source Control.
-->
<script lang="ts">
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import Logs from "./Logs.svelte";

  // TODO: Move Source Control to top
  const tabs = [
    { type: "logs", title: "Logs" } as const,
    { type: "sourceControl", title: "My Local Changes" } as const,
  ];

  let activeTab = tabs[0];
</script>

<Panel {tabs} bind:activeTab>
  <div class="tab" slot="tab" let:tab>
    <div class="title">
      <span>{tab.title}</span>
    </div>
  </div>
  <div class="content" slot="content">
    {#if activeTab.type === "sourceControl"}
      <div>Local Changes</div>
    {:else if activeTab.type === "logs"}
      <Logs />
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .tab {
    @apply flex flex-row justify-between h-full w-full;
  }

  .title {
    @apply flex flex-row items-center;
  }

  .content {
    @apply h-full w-full;
  }
</style>
