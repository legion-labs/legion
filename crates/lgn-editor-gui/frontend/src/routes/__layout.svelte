<script lang="ts" context="module">
  import type { Load } from "@sveltejs/kit";
  import { headlessRun } from "@lgn/web-client";
  import viewportOrchestrator from "@/orchestrators/viewport";
  import * as contextMenuEntries from "@/data/contextMenu";
  import contextMenu from "@/stores/contextMenu";
  import authStatus from "@/stores/authStatus";
  import type { NonEmptyArray } from "@lgn/web-client/src/lib/array";
  import log from "@lgn/web-client/src/lib/log";
  import { goto } from "$app/navigation";

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
    const editorServerUrl =
      url.searchParams.get("editor-server-url") || undefined;
    const runtimeServerUrl =
      url.searchParams.get("runtime-server-url") || undefined;

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

          contextMenu.register("resource", contextMenuEntries.resourceEntries);
          contextMenu.register(
            "resourcePanel",
            contextMenuEntries.resourcePanelEntries
          );

          const editorViewportKey = Symbol();

          viewportOrchestrator.addAllViewport(
            [editorViewportKey, { type: "video", name: "editor" }],
            [Symbol(), { type: "video", name: "runtime" }]
          );

          viewportOrchestrator.activate(editorViewportKey);
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
  import { initApiClient } from "@/api";
</script>

<slot />
