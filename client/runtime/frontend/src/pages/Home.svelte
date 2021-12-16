<script lang="ts">
  import Panel from "@/components/Panel.svelte";
  import TopBar from "@/components/TopBar.svelte";
  import StatusBar from "@/components/StatusBar.svelte";
  import Video, { Resolution } from "@/components/Video.svelte";

  let desiredVideoResolution: Resolution | null;
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
              backgroundColor="#112211"
              bind:desiredResolution={desiredVideoResolution}
            />
          </div>
        </Panel>
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
</style>
