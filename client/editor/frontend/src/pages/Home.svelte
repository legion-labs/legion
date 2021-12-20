<script lang="ts">
  import { getAllResources, getResourceProperties, ServerType } from "@/api";
  import CurrentResourceProperties from "@/components/CurrentResourceProperties.svelte";
  import { Panel, PanelList } from "@/components/panel";
  import TopBar from "@/components/TopBar.svelte";
  import StatusBar from "@/components/StatusBar.svelte";
  import RemoteWindow, { Resolution } from "@/components/RemoteWindow.svelte";
  import asyncData from "@/stores/asyncData";
  import currentResource from "@/stores/currentResource";
  import { ResourceDescription } from "@lgn/proto-editor/codegen/editor";
  import ScriptEditor from "@/components/ScriptEditor.svelte";

  const { run: runGetAllResources } = asyncData(getAllResources);

  let currentResourceDescription: ResourceDescription | null = null;
  let fetchAllResources = runGetAllResources();
  let desiredVideoResolution: Resolution | null;

  let editorActiveTab: ServerType;

  $: if (currentResourceDescription) {
    getResourceProperties(currentResourceDescription).then((resource) => {
      $currentResource = resource;
    });
  }

  function tryAgain() {
    $currentResource = null;
    currentResourceDescription = null;
    fetchAllResources = runGetAllResources();
  }

  function setCurrentResourceDescription(
    resourceDescription: ResourceDescription
  ) {
    currentResourceDescription = resourceDescription;
  }
</script>

<div class="root">
  <TopBar />
  <div class="content-wrapper">
    <div class="content">
      <div class="secondary-contents">
        <div class="resources">
          <Panel let:isFocused tabs={["Resources"]}>
            <div slot="tab" let:tab>{tab}</div>
            <div slot="content" class="resources-content">
              {#await fetchAllResources}
                <div class="resources-loading">Loading...</div>
              {:then resources}
                <PanelList
                  key="id"
                  items={resources}
                  activeItem={currentResourceDescription}
                  panelIsFocused={isFocused}
                  on:click={({ detail: resource }) =>
                    setCurrentResourceDescription(resource)}
                  on:itemChange={({ detail: { newItem: resource } }) =>
                    setCurrentResourceDescription(resource)}
                >
                  <div slot="default" let:item={resource}>
                    {resource.path}
                  </div>
                </PanelList>
              {:catch}
                <div class="resources-error">
                  An error occured while fetching the resources <span
                    class="resources-try-again"
                    on:click={tryAgain}
                  >
                    try again
                  </span>
                </div>
              {/await}
            </div>
          </Panel>
        </div>
        <div class="h-separator" />
        <div class="file-system">
          <Panel tabs={["File System"]}>
            <div slot="tab" let:tab>
              {tab}
            </div>
          </Panel>
        </div>
      </div>
      <div class="v-separator" />
      <div class="main-content">
        <Panel
          tabs={["editor", "runtime", "script"]}
          bind:activeTab={editorActiveTab}
          let:isFocused
        >
          <div slot="tab" let:tab>
            {#if tab === "editor" || tab === "runtime"}
              <span>{tab[0].toUpperCase()}{tab.slice(1)}</span>
              {#if desiredVideoResolution}
                <span>
                  - {desiredVideoResolution.width}x{desiredVideoResolution.height}
                </span>
              {/if}
            {:else if tab === "script"}
              Script
            {/if}
          </div>
          <div class="video-container" slot="content">
            {#if editorActiveTab === "editor" || editorActiveTab === "runtime"}
              {#key editorActiveTab}
                <RemoteWindow
                  serverType={editorActiveTab}
                  bind:desiredResolution={desiredVideoResolution}
                  {isFocused}
                />
              {/key}
            {:else if editorActiveTab === "script"}
              <ScriptEditor theme="vs-dark" />
            {/if}
          </div>
        </Panel>
      </div>
      <div class="v-separator" />
      <div class="secondary-contents">
        <div class="properties">
          <Panel tabs={["Properties"]}>
            <div slot="tab" let:tab>
              {tab}
            </div>
            <div slot="content">
              <CurrentResourceProperties />
            </div>
          </Panel>
        </div>
      </div>
    </div>
  </div>
  <StatusBar />
</div>

<style lang="postcss">
  .root {
    @apply h-screen w-full;
  }

  .content-wrapper {
    @apply h-[calc(100vh-3.5rem)] w-full overflow-auto;
  }

  .content {
    @apply flex flex-row h-full w-full;
  }

  .main-content {
    @apply flex flex-col w-full;
  }

  .video-container {
    @apply h-full w-full;
  }

  .v-separator {
    @apply flex-shrink-0 w-1;
  }

  .h-separator {
    @apply flex-shrink-0 h-1;
  }

  .secondary-contents {
    @apply flex flex-col flex-shrink-0 w-80 h-full;
  }

  .resources {
    @apply h-1/2;
  }

  .resources-loading {
    @apply px-2 py-1;
  }

  .resources-error {
    @apply px-2 py-1;
  }

  .resources-try-again {
    @apply underline text-blue-300 cursor-pointer;
  }

  .resources-content {
    @apply h-full break-all;
  }

  .file-system {
    @apply h-1/2;
  }

  .properties {
    @apply h-full;
  }
</style>
