<script lang="ts" context="module">
  import { goto } from "$app/navigation";
  import type { Load } from "@sveltejs/kit";

  import { headlessRun } from "@lgn/web-client";
  import type { NonEmptyArray } from "@lgn/web-client/src/lib/array";
  import log from "@lgn/web-client/src/lib/log";
  import {
    createPanel,
    createTile,
  } from "@lgn/web-client/src/stores/workspace";

  import { initApiClient } from "@/api";
  import * as contextMenuEntries from "@/data/contextMenu";
  import { initMessageStream } from "@/orchestrators/selection";
  import authStatus from "@/stores/authStatus";
  import contextMenu from "@/stores/contextMenu";
  import { initLogStream } from "@/stores/log";
  import { initStagedResourcesStream } from "@/stores/stagedResources";
  import tabPayloads from "@/stores/tabPayloads";
  import workspace, {
    viewportPanelId,
    viewportTileId,
  } from "@/stores/workspace";
  import type { TabType } from "@/stores/workspace";
  import "@/workers/editorWorker";

  const logLevel = "warn";

  const scopes: NonEmptyArray<string> = [
    "aws.cognito.signin.user.admin",
    "email",
    "https://legionlabs.com/editor/allocate",
    "openid",
    "profile",
  ];

  const issuerUrl =
    "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz";

  const clientId = "5m58nrjfv6kr144prif9jk62di";

  const redirectUri = `${document.location.origin}/`;

  export const load: Load = async ({ fetch, url }) => {
    const editorServerUrlKey = "editor-server-url";
    const editorRuntimerUrlKey = "editor-runtime-url";

    if (url.searchParams.has(editorServerUrlKey)) {
      sessionStorage.setItem(
        editorServerUrlKey,
        url.searchParams.get(editorServerUrlKey) as string
      );
    }

    if (url.searchParams.has(editorRuntimerUrlKey)) {
      sessionStorage.setItem(
        editorRuntimerUrlKey,
        url.searchParams.get(editorRuntimerUrlKey) as string
      );
    }

    const editorServerUrl =
      sessionStorage.getItem(editorServerUrlKey) || undefined;

    const runtimeServerUrl =
      sessionStorage.getItem(editorRuntimerUrlKey) || undefined;

    initApiClient({ editorServerUrl });

    try {
      const { initAuthStatus } = await headlessRun({
        auth: {
          fetch,
          issuerUrl,
          redirectUri,
          clientId,
          url,
          redirectFunction(url) {
            return goto(url, { replaceState: true });
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
        logLevel,
        async onPreInit() {
          // await initWasmLogger();
          // debug("Hello from the Legion editor");

          // Fire and forget stream init
          // TODO: When using routing we may want to cancel the returned subscription
          initLogStream();

          // Fire and forget stream init
          // TODO: When using routing we may want to cancel the returned subscription
          initMessageStream();

          // Fire and forget stream init
          // TODO: When using routing we may want to cancel the returned subscription
          initStagedResourcesStream();

          contextMenu.register("resource", contextMenuEntries.resourceEntries);
          contextMenu.register(
            "resourcePanel",
            contextMenuEntries.resourcePanelEntries
          );

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

          workspace.pushTile(viewportTile);
        },
      });

      authStatus.set(initAuthStatus);

      return {};
    } catch (error) {
      log.error("Application couldn't start", error);

      return { status: 500 };
    }
  };
</script>

<script lang="ts">
  import "../assets/index.css";
</script>

<slot />
