<script lang="ts">
  import { getAllResources, getResourceProperties } from "@/api";
  import Panel from "@/components/Panel.svelte";
  import TopBar from "@/components/TopBar.svelte";
  import Video, {
    Resolution,
    ResourceWithProperties,
  } from "@/components/Video.svelte";
  import { asyncData } from "@/stores/asyncData";

  let selectedResourceId: string | null = null;
  let selectedResource: ResourceWithProperties | null = null;

  const { run } = asyncData(getAllResources);

  let fetchAllResources = run();

  let desiredVideoResolution: Resolution | undefined;

  const tryAgain = () => {
    fetchAllResources = run();
  };

  $: if (selectedResourceId) {
    getResourceProperties(selectedResourceId).then(
      ({ description, properties }) => {
        if (description) {
          selectedResource = { description, properties };
        }
      }
    );
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
            <Video
              bind:desiredResolution={desiredVideoResolution}
              resource={selectedResource}
            />
          </div>
        </Panel>
      </div>
      <div class="v-separator" />
      <div class="secondary-contents">
        <div class="resources">
          <Panel>
            <span slot="header"> Resources </span>
            <div class="resource-content" slot="content">
              {#await fetchAllResources}
                <div class="resources-loading">loading...</div>
              {:then data}
                {#each data as resource (resource.id)}
                  <div
                    class="resource-item"
                    class:active-resource-item={selectedResourceId ===
                      resource.id}
                    on:click={() => (selectedResourceId = resource.id)}
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
        <Panel>
          <span slot="header"> Properties </span>
          <span slot="content"> <div /> </span>
        </Panel>
      </div>
    </div>
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-screen w-full;
  }

  .content-wrapper {
    @apply h-[calc(100vh-1.75rem)] w-full;
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
    @apply pb-2;
  }

  .resource-item {
    @apply cursor-pointer hover:bg-gray-400 py-1 px-2;
  }

  .active-resource-item {
    @apply bg-gray-400 italic;
  }
</style>
