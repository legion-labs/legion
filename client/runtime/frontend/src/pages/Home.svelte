<script lang="ts">
  import { Panel } from "@lgn/frontend/src/components/panel";
  import TopBar from "@lgn/frontend/src/components/TopBar.svelte";
  import StatusBar from "@lgn/frontend/src/components/StatusBar.svelte";
  import RemoteWindow from "@lgn/frontend/src/components/RemoteWindow.svelte";
  import { Resolution } from "@lgn/frontend/src/lib/types";

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
            <RemoteWindow
              serverType="runtime"
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
