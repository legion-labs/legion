<script lang="ts">
  import Icon from "@iconify/svelte";
  import Panel from "./Panel.svelte";
  import RemoteWindow from "../RemoteWindow.svelte";
  import ScriptEditor from "../ScriptEditor.svelte";
  import ViewportOrchestrator from "../../stores/viewport";
  import { Resolution } from "../../lib/types";

  /** The global viewport orchestrator */
  export let orchestrator: ViewportOrchestrator;

  let desiredVideoResolution: Resolution | null;

  $: viewportStore = orchestrator.viewportStore;

  $: activeViewportStore = orchestrator.activeViewportStore;
</script>

<Panel
  tabs={Array.from($viewportStore.values())}
  bind:activeTab={$activeViewportStore}
>
  <div class="tab" slot="tab" let:tab>
    <div class="title">
      {#if tab.type === "video"}
        <span>{tab.name[0].toUpperCase()}{tab.name.slice(1)}</span>
        {#if desiredVideoResolution}
          <span>
            - {desiredVideoResolution.width}x{desiredVideoResolution.height}
          </span>
        {/if}
      {:else if tab.type === "script"}
        {tab.name}
      {/if}
    </div>
    {#if tab.removable}
      <div class="close" on:click={() => orchestrator.removeByValue(tab)}>
        <Icon icon="ic:baseline-close" />
      </div>
    {/if}
  </div>
  <div class="content" slot="content" let:activeTab>
    {#if activeTab.type === "video"}
      {#if activeTab.name === "editor" || activeTab.name === "runtime"}
        {#key activeTab}
          <RemoteWindow
            serverType={activeTab.name}
            bind:desiredResolution={desiredVideoResolution}
          />
        {/key}
      {/if}
    {:else if activeTab.type === "script"}
      <ScriptEditor
        theme="vs-dark"
        on:change={({ detail: newValue }) =>
          activeTab.type === "script" && activeTab.onChange(newValue)}
        value={activeTab.value}
      />
    {/if}
  </div>
</Panel>

<style lang="postcss">
  .tab {
    @apply flex flex-row justify-between space-x-4 h-full w-full;

    .title {
      @apply flex flex-row items-center;
    }

    .close {
      @apply flex flex-row justify-center items-center cursor-pointer text-orange-700;
    }
  }

  .content {
    @apply h-full w-full;
  }
</style>
