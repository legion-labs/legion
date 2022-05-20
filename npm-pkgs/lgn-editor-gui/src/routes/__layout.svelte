<script lang="ts" context="module">
  import { goto } from "$app/navigation";
  import type { Load } from "@sveltejs/kit";
  import { get } from "svelte/store";

  import { headlessRun } from "@lgn/web-client";
  import type { NonEmptyArray } from "@lgn/web-client/src/lib/array";
  import log from "@lgn/web-client/src/lib/log";
  import type { Level } from "@lgn/web-client/src/lib/log";
  import { ConsoleTransport } from "@lgn/web-client/src/lib/log/transports";
  import {
    createEmptyPanel,
    createPanel,
    createTile,
  } from "@lgn/web-client/src/stores/workspace";

  import { initApiClient } from "@/api";
  import { fetchAllActiveScenes } from "@/orchestrators/allActiveScenes";
  import contextMenu, {
    localChangesContextMenuId,
    localChangesEntries,
    resourceBrowserItemContextMenuId,
    resourceBrowserItemEntries,
    resourceBrowserPanelContextMenuId,
    resourceBrowserPanelEntries,
  } from "@/stores/contextMenu";
  import tabPayloads from "@/stores/tabPayloads";
  import workspace, {
    sceneExplorerPanelId,
    sceneExplorerTileId,
    viewportPanelId,
    viewportTileId,
  } from "@/stores/workspace";
  import type { TabType } from "@/stores/workspace";
  import "@/workers/editorWorker";

  const scopes: NonEmptyArray<string> = [
    "aws.cognito.signin.user.admin",
    "email",
    "https://legionlabs.com/editor/allocate",
    "openid",
    "profile",
  ];

  const issuerUrl = import.meta.env
    .VITE_ONLINE_AUTHENTICATION_OAUTH_ISSUER_URL as string;

  const clientId = import.meta.env
    .VITE_ONLINE_AUTHENTICATION_OAUTH_CLIENT_ID as string;

  const redirectUri = `${document.location.origin}/`;

  export const load: Load = async ({ fetch, url }) => {
    const editorServerUrlKey = "editor-server-url";
    const runtimeServerUrlKey = "runtime-server-url";

    devSettings.update((devSettings) => ({
      ...devSettings,
      editorServerUrl:
        url.searchParams.get(editorServerUrlKey) || devSettings.editorServerUrl,
      runtimeServerUrl:
        url.searchParams.get(runtimeServerUrlKey) ||
        devSettings.runtimeServerUrl,
    }));

    const { editorServerUrl, runtimeServerUrl } = get(devSettings);

    initApiClient({ editorServerUrl, runtimeServerUrl });

    try {
      const { dispose, initAuthStatus } = await headlessRun({
        auth: {
          fetch,
          issuerUrl,
          redirectUri,
          clientId,
          url,
          redirectFunction(url) {
            return goto(url.toString(), { replaceState: true });
          },
          login: {
            cookies: {
              accessToken: "editor_access_token",
              refreshToken: "editor_refresh_token",
            },
            extraParams: {
              // eslint-disable-next-line camelcase
              identity_provider: "Azure",
            },
            scopes,
          },
        },
        editorServerUrl,
        runtimeServerUrl,
        log: {
          transports: [
            new ConsoleTransport({
              level: import.meta.env.VITE_CONSOLE_LOG_LEVEL as Level,
            }),
          ],
        },
        onPreInit() {
          // await initWasmLogger();
          // debug("Hello from the Legion editor");

          contextMenu.register(
            resourceBrowserItemContextMenuId,
            resourceBrowserItemEntries
          );

          contextMenu.register(
            resourceBrowserPanelContextMenuId,
            resourceBrowserPanelEntries
          );

          contextMenu.register(localChangesContextMenuId, localChangesEntries);

          const videoEditorTabPayloadId = "video-editor-payload";

          const videoRuntimeTabPayloadId = "video-runtime-payload";

          tabPayloads.update((tabPayloads) => ({
            ...tabPayloads,
            [videoEditorTabPayloadId]: {
              type: "video",
              serverType: "editor",
            },
          }));

          tabPayloads.update((tabPayloads) => ({
            ...tabPayloads,
            [videoRuntimeTabPayloadId]: {
              type: "video",
              serverType: "runtime",
            },
          }));

          const viewportTile = createTile<TabType>(
            viewportTileId,
            createPanel<TabType>(viewportPanelId, [
              {
                id: "editor-main",
                type: "video",
                label: "Editor",
                payloadId: videoEditorTabPayloadId,
              },
              {
                id: "runtime-main",
                type: "video",
                label: "Runtime",
                payloadId: videoRuntimeTabPayloadId,
              },
            ]),
            { trackSize: false }
          );

          const sceneExplorerTile = createTile<TabType>(
            sceneExplorerTileId,
            createEmptyPanel<TabType>(sceneExplorerPanelId),
            { trackSize: false }
          );

          workspace.appendAllTiles([viewportTile, sceneExplorerTile]);
        },
      });

      authStatus.set(initAuthStatus);

      return { props: { dispose } };
    } catch (error) {
      log.error("Application couldn't start", error);

      return { status: 500 };
    }
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";

  import ContextMenu from "@lgn/web-client/src/components/ContextMenu.svelte";
  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import ModalContainer from "@lgn/web-client/src/components/modal/ModalContainer.svelte";

  import AuthModal from "@/components/AuthModal.svelte";
  import { fileName } from "@/lib/path";
  import { allActiveScenes } from "@/orchestrators/allActiveScenes";
  import { initMessageStream } from "@/orchestrators/selection";
  import authStatus from "@/stores/authStatus";
  import devSettings from "@/stores/devSettings";
  import { initLogStreams } from "@/stores/log";
  import modal from "@/stores/modal";
  import notifications from "@/stores/notifications";

  import "../assets/index.css";

  export let dispose: (() => void) | undefined;

  onMount(async () => {
    if ($authStatus && $authStatus.type === "error") {
      modal.open(Symbol.for("auth-modal"), AuthModal, {
        payload: { authorizationUrl: $authStatus.authorizationUrl },
        noTransition: true,
      });
    }

    const initLogStreamSubscriptions = initLogStreams();
    const initMessageStreamSubscription = initMessageStream();
    // const initStagedResourcesStreamSubscription = await initStagedResourcesStream();

    // TODO: Reactivate when the streaming is ready server-side
    // initAllActiveScenesStream();
    await fetchAllActiveScenes();

    const allActiveScenesSubscription = allActiveScenes.subscribe(
      (allActiveScenes) => {
        if (!allActiveScenes) {
          tabPayloads.update((tabPayloads) =>
            Object.fromEntries(
              Object.entries(tabPayloads).filter(
                ([_id, payload]) => payload.type !== "sceneExplorer"
              )
            )
          );

          workspace.setPanelTabs(sceneExplorerPanelId, null);
        } else {
          tabPayloads.update((tabPayloads) => {
            const cleanedTabPayloads = Object.fromEntries(
              Object.entries(tabPayloads).filter(
                ([_id, payload]) => payload.type !== "sceneExplorer"
              )
            );

            allActiveScenes.forEach(({ rootScene }) => {
              cleanedTabPayloads[rootScene.id] = {
                type: "sceneExplorer",
                rootSceneId: rootScene.id,
              };
            });

            return cleanedTabPayloads;
          });

          workspace.setPanelTabs(
            sceneExplorerPanelId,
            allActiveScenes.map(({ rootScene }) => ({
              id: rootScene.id,
              payloadId: rootScene.id,
              label: fileName(rootScene.path),
              type: "sceneExplorer",
              disposable: true,
            })) as NonEmptyArray<TabType>,
            { activeTabIndex: allActiveScenes.length - 1 }
          );
        }
      }
    );

    return () => {
      dispose?.();
      initLogStreamSubscriptions();
      initMessageStreamSubscription();
      // initStagedResourcesStreamSubscription();
      // TODO: Uncomment when the streaming is ready server-side
      // initAllActiveScenesStream();
      allActiveScenesSubscription();
    };
  });
</script>

<ModalContainer store={modal} />

<ContextMenu store={contextMenu} />

<Notifications store={notifications} />

<slot />
