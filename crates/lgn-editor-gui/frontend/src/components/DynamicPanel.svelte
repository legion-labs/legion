<script lang="ts">
  import Icon from "@iconify/svelte";

  import RemoteWindow from "@lgn/web-client/src/components/RemoteWindow.svelte";
  import Panel from "@lgn/web-client/src/components/panel/Panel.svelte";
  import type { Resolution } from "@lgn/web-client/src/lib/types";
  import type { Panel as WorkspacePanel } from "@lgn/web-client/src/stores/workspace";

  import { closeScene } from "@/api";
  import { fetchAllActiveScenes } from "@/orchestrators/allActiveScenes";
  import type { TabPayload } from "@/stores/tabPayloads";
  import tabPayloads from "@/stores/tabPayloads";
  import type { TabType } from "@/stores/workspace";
  import workspace from "@/stores/workspace";

  import SceneExplorerTab from "./tabs/SceneExplorerTab.svelte";
  import ScriptTab from "./tabs/ScriptTab.svelte";

  export let panel: Exclude<WorkspacePanel<TabType>, { type: "emptyPanel" }>;

  let desiredResolution: Resolution | null;

  $: activeTab = panel.tabs[panel.activeTabIndex];

  $: payload = activeTab?.payloadId ? $tabPayloads[activeTab?.payloadId] : null;

  function updatePayload({ detail: newPayload }: CustomEvent<TabPayload>) {
    if (activeTab?.payloadId) {
      $tabPayloads[activeTab.payloadId] = newPayload;
    }
  }

  async function closeTab(tab: TabType) {
    workspace.removeTabFromPanelByValue(panel.id, tab);

    if (tab.payloadId) {
      const { [tab.payloadId]: removedTabPayload, ...remainingTabPayloads } =
        $tabPayloads;

      // TODO: Move away
      if (
        tab.type === "sceneExplorer" &&
        removedTabPayload.type === "sceneExplorer"
      ) {
        await closeScene({ id: removedTabPayload.rootSceneId });

        await fetchAllActiveScenes();
      }

      $tabPayloads = remainingTabPayloads;
    }
  }
</script>

<Panel
  tabs={Array.from(panel.tabs.values())}
  bind:activeTabIndex={panel.activeTabIndex}
>
  <div class="tab" slot="tab" let:tab>
    <div class="title">
      {#if tab.type === "video"}
        <span>{tab.label}</span>
        <!-- TODO: Move out -->
        {#if desiredResolution}
          <span>
            - {desiredResolution.width}x{desiredResolution.height}
          </span>
        {/if}
      {:else if tab.type === "script"}
        {tab.label}
      {:else if tab.type === "sceneExplorer"}
        {tab.label}
      {/if}
    </div>
    {#if tab.disposable}
      <div class="close" on:click={() => closeTab(tab)}>
        <Icon icon="ic:baseline-close" />
      </div>
    {/if}
  </div>
  <div class="content" slot="content">
    {#if !activeTab}
      <div />
    {:else if activeTab.type === "video" && payload?.type === "video"}
      <!-- TODO: Use ViewportTab -->
      {#if payload.serverType === "editor" || payload.serverType === "runtime"}
        {#key payload.serverType}
          <RemoteWindow
            serverType={payload.serverType}
            bind:desiredResolution
          />
        {/key}
      {/if}
    {:else if activeTab.type === "script" && payload?.type === "script"}
      <ScriptTab payloadId={activeTab.id} {payload} on:change={updatePayload} />
    {:else if activeTab.type === "sceneExplorer" && payload?.type === "sceneExplorer"}
      <SceneExplorerTab payloadId={activeTab.id} {payload} />
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .tab {
    @apply flex flex-row justify-between space-x-4 h-full w-full;
  }

  .title {
    @apply flex flex-row items-center;
  }

  .close {
    @apply flex flex-row justify-center items-center cursor-pointer text-orange-700;
  }

  .content {
    @apply h-full w-full;
  }
</style>
