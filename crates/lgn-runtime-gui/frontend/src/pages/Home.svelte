<script lang="ts">
  import { Panel } from "@lgn/web-client/src/components/panel";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import RemoteWindow from "@lgn/web-client/src/components/RemoteWindow.svelte";
  import { Resolution } from "@lgn/web-client/src/lib/types";

  let desiredVideoResolution: Resolution | null;
</script>

<div class="root">
  <TopBar />
  <div class="content-wrapper">
    <div class="content">
      <div class="main-content">
        <Panel tabs={["Main Stream"]}>
          <div class="tab" slot="tab" let:tab>
            <div class="title">
              <span>{tab[0].toUpperCase()}{tab.slice(1)}</span>
              {#if desiredVideoResolution}
                <span>
                  - {desiredVideoResolution.width}x{desiredVideoResolution.height}
                </span>
              {/if}
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

  .content-wrapper {
    @apply h-[calc(100vh-4rem)] w-full overflow-auto;
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
