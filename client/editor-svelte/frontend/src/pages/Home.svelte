<script lang="ts">
  import { getAllResources, getResourceProperties } from "@/api";
  import CurrentResourceProperties from "@/components/CurrentResourceProperties.svelte";
  import Panel from "@/components/Panel.svelte";
  import TopBar from "@/components/TopBar.svelte";
  import StatusBar from "@/components/StatusBar.svelte";
  import Video, { Resolution } from "@/components/Video.svelte";
  import asyncData from "@/stores/asyncData";
  import currentResource from "@/stores/currentResource";

  const { run } = asyncData(getAllResources);

  let currentResourceId: string | null = null;
  let fetchAllResources = run();
  let desiredVideoResolution: Resolution | undefined;

  const tryAgain = () => {
    $currentResource = null;
    currentResourceId = null;
    fetchAllResources = run();
  };

  $: if (currentResourceId) {
    getResourceProperties(currentResourceId).then((resource) => {
      $currentResource = resource;
    });
  }
</script>

<div class="root">
  <TopBar />
  <div class="content-wrapper">
    <div class="content">
      <div class="main-content">
        <Panel>
          <span slot="header">
            <span>Main Stream</span>
            {#if desiredVideoResolution}
              <span>
                - {desiredVideoResolution.width}x{desiredVideoResolution.height}
              </span>
            {/if}
          </span>
          <div class="video-container" slot="content">
            <Video bind:desiredResolution={desiredVideoResolution} />
          </div>
        </Panel>
      </div>
      <div class="v-separator" />
      <div class="secondary-contents">
        <div class="resources">
          <Panel>
            <span slot="header">Resources</span>
            <div class="resource-content" slot="content">
              {#await fetchAllResources}
                <div class="resources-loading">loading...</div>
              {:then data}
                {#each data as resource (resource.id)}
                  <div
                    class="resource-item"
                    class:active-resource-item={currentResourceId ===
                      resource.id}
                    on:click={() => (currentResourceId = resource.id)}
                  >
                    {resource.path}
                  </div>
                {/each}
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
        <div class="properties">
          <Panel>
            <div slot="header">Properties</div>
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
    @apply flex flex-col flex-shrink-0 w-3/4;
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
    @apply flex flex-col w-full;
  }

  .resources {
    @apply h-1/3;
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

  .resource-content {
    @apply pb-2 break-all;
  }

  .resource-item {
    @apply cursor-pointer hover:bg-gray-500 py-1 px-2;
  }

  .active-resource-item {
    @apply bg-gray-500 italic;
  }

  .properties {
    @apply h-full overflow-hidden;
  }
</style>
