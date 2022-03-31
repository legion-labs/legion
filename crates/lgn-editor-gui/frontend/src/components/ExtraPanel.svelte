<!--
@Component
Contains all the "extra" panels like Log or Source Control.
-->
<script lang="ts">
  import { Panel } from "@lgn/web-client/src/components/panel";

  import Log from "./Log.svelte";
  import LocalChanges from "./localChanges/LocalChanges.svelte";

  const tabs = [
    { type: "sourceControl", title: "My Local Changes" } as const,
    { type: "log", title: "Log" } as const,
  ];
</script>

<Panel {tabs}>
  <div class="tab" slot="tab" let:tab>
    <div class="title">
      <span>{tab.title}</span>
    </div>
  </div>
  <div class="content" slot="content" let:activeTab>
    {#if activeTab.type === "sourceControl"}
      <LocalChanges />
    {:else if activeTab.type === "log"}
      <Log />
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
    @apply flex flex-col h-full w-full;
  }
</style>
