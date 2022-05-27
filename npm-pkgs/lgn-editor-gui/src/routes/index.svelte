<script lang="ts" context="module">
  export type RootContext = {
    getLayout: () => Layout;
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { setContext } from "svelte";

  import Log from "@lgn/web-client/src/components/Log.svelte";
  import RemoteWindow from "@lgn/web-client/src/components/RemoteWindow.svelte";
  import StatusBar from "@lgn/web-client/src/components/StatusBar.svelte";
  import Tile from "@lgn/web-client/src/components/Tile.svelte";
  import TopBar from "@lgn/web-client/src/components/TopBar.svelte";
  import Layout from "@lgn/web-client/src/components/layout/Layout.svelte";
  import type { MenuItemDescription } from "@lgn/web-client/src/components/menu/lib/MenuItemDescription";
  import { EmptyPanel, Panel } from "@lgn/web-client/src/components/panel";

  import { closeScene } from "@/api";
  import DynamicPanel from "@/components/DynamicPanel.svelte";
  import ExtraPanel from "@/components/ExtraPanel.svelte";
  import ResourceBrowser from "@/components/ResourceBrowser.svelte";
  import SceneExplorer from "@/components/SceneExplorer.svelte";
  import {
    AppComponentMap as appComponentMap,
    defaultLayoutConfig,
  } from "@/components/layout/LayoutConfig";
  import LocalChanges from "@/components/localChanges/LocalChanges.svelte";
  import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
  import { fileName } from "@/lib/path";
  import {
    allActiveScenes,
    fetchAllActiveScenes,
  } from "@/orchestrators/allActiveScenes";
  import {
    allResourcesError,
    fetchAllResources,
  } from "@/orchestrators/allResources";
  import { currentResource } from "@/orchestrators/currentResource";
  import { currentResourceDescriptionEntry } from "@/orchestrators/resourceBrowserEntries";
  import devSettings from "@/stores/devSettings";
  import { stagedResources, syncFromMain } from "@/stores/stagedResources";
  import workspace, { sceneExplorerTileId } from "@/stores/workspace";
  import { viewportTileId } from "@/stores/workspace";

  $: if ($allResourcesError) {
    refetchResources().catch(() => {
      // TODO: Handle errors
    });
  }

  onMount(async () => {
    refetchResources().catch(() => {
      // TODO: Handle errors
    });

    await fetchAllActiveScenes();

    return allActiveScenes.subscribe((scenes) => {
      scenes?.forEach((s) => {
        layout.addComponent(
          SceneExplorer.name,
          {
            state: {
              activeScenes: s.scenes,
            },
            onDestroyed: async () => {
              await closeScene({ id: s.rootScene.id });
              await fetchAllActiveScenes();
            },
          },
          fileName(s.rootScene.path) ?? "undefined",
          s.rootScene.id
        );
      });
    });
  });

  function refetchResources() {
    $currentResource = null;
    $currentResourceDescriptionEntry = null;

    return fetchAllResources();
  }

  const mainMenuItemDescriptions: MenuItemDescription[] = [
    {
      title: "Window",
      visible: true,
      children: [
        {
          title: "Editor",
          visible: true,
          action: () => {
            layout.addComponent(
              RemoteWindow.name,
              {
                state: {
                  serverType: "editor",
                },
              },
              "Editor"
            );
          },
        },
        {
          title: "Runtime",
          visible: true,

          action: () => {
            layout.addComponent(
              RemoteWindow.name,
              {
                state: {
                  serverType: "runtime",
                },
              },
              "Runtime"
            );
          },
        },
        {
          title: "Property Grid",
          visible: true,

          action: () => {
            layout.addComponent(PropertyGrid.name);
          },
        },
        {
          title: "Resource Browser",
          visible: true,

          action: () => {
            layout.addComponent(ResourceBrowser.name);
          },
        },
        {
          title: "Local Changes",
          visible: true,

          action: () => {
            layout.addComponent(LocalChanges.name);
          },
        },
        {
          title: "Logs",
          visible: true,
          action: () => {
            layout.addComponent(Log.name);
          },
        },
      ],
    },
  ];

  let layout: Layout;

  setContext<RootContext>("root", {
    getLayout() {
      return layout;
    },
  });

  const enableOld = false;
</script>

<div class="root">
  <TopBar devSettings={$devSettings} {mainMenuItemDescriptions} />
  <div class="content-wrapper" class:electron={window.isElectron}>
    <div class="content">
      <Layout
        surfaceClass="700"
        layoutConfig={defaultLayoutConfig}
        componentMap={appComponentMap}
        bind:this={layout}
      />
      {#if enableOld}
        <div class="secondary-contents">
          <div class="scene-explorer">
            <!-- TODO: Move this into a dedicated component DynamicTile -->
            <Tile id={sceneExplorerTileId} {workspace}>
              <div class="h-full w-full" slot="default" let:tile>
                {#if tile?.panel?.type === "populatedPanel"}
                  <DynamicPanel panel={tile.panel} />
                {:else}
                  <EmptyPanel>
                    <div class="empty-panel">
                      <em>No open scenes</em>
                    </div>
                  </EmptyPanel>
                {/if}
              </div>
            </Tile>
          </div>
          <div class="h-separator" />
          <div class="resource-browser">
            <ResourceBrowser />
          </div>
        </div>
        <div class="v-separator" />
        <div class="main-content">
          <!-- TODO: Move this into a dedicated component DynamicTile -->
          <Tile id={viewportTileId} {workspace}>
            <div class="h-full w-full" slot="default" let:tile>
              {#if tile?.panel?.type === "populatedPanel"}
                <DynamicPanel panel={tile.panel} />
              {:else}
                <EmptyPanel>
                  <div class="empty-panel">
                    <em>No open videos</em>
                  </div>
                </EmptyPanel>
              {/if}
            </div>
          </Tile>
          <div class="h-separator" />
          <div class="extra-panel">
            <ExtraPanel />
          </div>
        </div>
        <div class="v-separator" />
        <div class="secondary-contents">
          <div class="property-grid">
            <Panel tabs={["Property Grid"]}>
              <div slot="tab" let:tab>
                {tab}
              </div>
              <div class="property-grid-content" slot="content">
                <PropertyGrid />
              </div>
            </Panel>
          </div>
        </div>
      {/if}
    </div>
  </div>
  <StatusBar {syncFromMain} stagedResources={$stagedResources || []} />
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

  .v-separator {
    @apply flex-shrink-0 w-1;
  }

  .h-separator {
    @apply flex-shrink-0 h-1;
  }

  .secondary-contents {
    @apply flex flex-col flex-shrink-0 w-96 h-full;
  }

  .scene-explorer {
    @apply h-[calc(50%-theme("spacing[0.5]"))];
  }

  .resource-browser {
    @apply h-[calc(50%-theme("spacing[0.5]"))];
  }

  .property-grid {
    @apply h-full;
  }

  .property-grid-content {
    @apply h-full;
  }

  .extra-panel {
    @apply h-80 flex-shrink-0;
  }

  .empty-panel {
    @apply flex items-center justify-center h-full w-full text-xl font-bold;
  }
</style>
