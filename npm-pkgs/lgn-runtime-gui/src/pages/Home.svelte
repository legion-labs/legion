<script lang="ts">
  import RemoteWindow from "@lgn/web-client/src/components/RemoteWindow.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import { Panel } from "@lgn/web-client/src/components/panel";
  import type { Resolution } from "@lgn/web-client/src/lib/types";

  let desiredVideoResolution: Resolution | null;
</script>

<div class="root">
  <TopBar />
  <div class="content-wrapper" class:electron={window.isElectron}>
    <div class="content">
      <div class="main-content">
        <Panel tabs={["Main Stream"]}>
          <div class="tab" slot="tab" let:tab>
            <div class="title">
              <span>{tab[0]?.toUpperCase()}{tab.slice(1)}</span>
            </div>
          </div>
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

  .root .content-wrapper {
    @apply h-[calc(100vh-4.5rem)] w-full overflow-auto;
  }

  .root .content-wrapper.electron {
    @apply h-[calc(100vh-5rem)];
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
