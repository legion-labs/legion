<script lang="ts" context="module">
  import { goto } from "$app/navigation";
  import type { Load } from "@sveltejs/kit";

  import { headlessRun } from "@lgn/web-client";
  import { resolveIssuerUrl } from "@lgn/web-client/src/lib/auth/cognito";
  import type { Level } from "@lgn/web-client/src/lib/log";
  import {
    ConsoleTransport,
    NotificationsTransport,
  } from "@lgn/web-client/src/lib/log/transports";
  import { createNotificationsStore } from "@lgn/web-client/src/stores/notifications";

  import { getRuntimeConfig } from "../lib/runtimeConfig";

  const redirectUri = document.location.origin + "/";

  const notifications = createNotificationsStore<Fluent>();

  const runtimeConfig = getRuntimeConfig();

  const issuerUrl = resolveIssuerUrl({
    region: runtimeConfig?.cognitoRegion ?? "",
    poolId: runtimeConfig?.cognitoPoolId ?? "",
  });

  let loaded = false;

  export const load: Load = async ({ fetch, url }) => {
    if (loaded) {
      return { status: 200 };
    }

    try {
      const { dispose, initAuthStatus } = await headlessRun({
        auth: {
          fetch: fetch as typeof globalThis.fetch,
          issuerUrl,
          redirectUri,
          clientId: runtimeConfig?.clientId ?? "",
          login: {
            cookies: {
              accessToken: accessTokenCookieName,
              refreshToken: refreshTokenCookieName,
            },
            scopes: ["email", "openid", "profile"],
          },
          redirectFunction(url) {
            return goto(url.toString(), { replaceState: true });
          },
          url,
        },
        log: {
          transports: [
            new ConsoleTransport({
              level: import.meta.env
                .VITE_LEGION_ANALYTICS_CONSOLE_LOG_LEVEL as Level,
            }),
            new NotificationsTransport<Fluent>({
              notificationsStore: notifications,
              level: import.meta.env
                .VITE_LEGION_ANALYTICS_NOTIFICATION_LOG_LEVEL as Level,
            }),
          ],
        },
      });

      loaded = true;

      return {
        props: { dispose, initAuthStatus, notifications, runtimeConfig },
      };
    } catch (error) {
      log.error("Application couldn't start", error);

      return { status: 500 };
    }
  };
</script>

<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { writable } from "svelte/store";

  import Notifications from "@lgn/web-client/src/components/Notifications.svelte";
  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";
  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth/index";
  import { displayError } from "@lgn/web-client/src/lib/errors";
  import { replaceClassesWith } from "@lgn/web-client/src/lib/html";
  import log from "@lgn/web-client/src/lib/log";
  import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
  import { createL10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";
  import type { NotificationsStore } from "@lgn/web-client/src/stores/notifications";
  import { createThemeStore } from "@lgn/web-client/src/stores/theme";

  import en from "@/assets/locales/en-US/main.ftl?raw";
  import fr from "@/assets/locales/fr-CA/main.ftl?raw";
  import LoadingBar from "@/components/Misc/LoadingBar.svelte";
  import { getThreadItemLength } from "@/components/Timeline/Values/TimelineValues";
  import {
    accessTokenCookieName,
    localeStorageKey,
    refreshTokenCookieName,
    themeStorageKey,
    threadItemLengthFallback,
  } from "@/constants";
  import { createGrpcClient } from "@/lib/client";
  import type { RuntimeConfig } from "@/lib/runtimeConfig";

  import "../assets/index.css";

  export let initAuthStatus: InitAuthStatus | null;

  export let notifications: NotificationsStore<Fluent>;

  export let runtimeConfig: RuntimeConfig;

  export let dispose: () => void | undefined;

  const theme = createThemeStore(themeStorageKey, "dark");

  const l10n = createL10nOrchestrator<Fluent>(
    [
      {
        names: ["en-US", "en"],
        contents: [en],
      },
      {
        names: ["fr-CA", "fr"],
        contents: [fr],
      },
    ],
    {
      local: {
        functions: {
          LOWERCASE([value]) {
            if (!value || typeof value !== "string") {
              return value;
            }

            return value.toLowerCase();
          },
        },
        connect: {
          key: localeStorageKey,
          storage: new DefaultLocalStorage(),
        },
      },
    }
  );

  setContext("runtime-config", runtimeConfig);

  setContext("theme", theme);

  setContext(l10nOrchestratorContextKey, l10n);

  setContext("http-client", createGrpcClient(runtimeConfig.apiAnalytics.url));

  setContext("notifications", notifications);

  setContext(
    "debug",
    writable(import.meta.env.VITE_LEGION_ANALYTICS_DEBUG === "true")
  );

  try {
    setContext("thread-item-length", getThreadItemLength());
  } catch (error) {
    log.warn(
      `Couldn't get the proper thread item length, defaulting to the arbitrary value "${threadItemLengthFallback}": ${displayError(
        error
      )}`
    );

    setContext("thread-item-length", threadItemLengthFallback);
  }

  // TODO: Here we can control the UI and display a modal or change the page content
  onMount(() => {
    if (initAuthStatus?.type === "error") {
      window.location.href = initAuthStatus.authorizationUrl;
    }

    const unsubscribe = theme.subscribe(({ name }) => {
      replaceClassesWith(document.body, `theme-${name}`);
    });

    return () => {
      dispose?.();

      unsubscribe();
    };
  });
</script>

{#if initAuthStatus?.type !== "error"}
  <Notifications store={notifications} />

  <LoadingBar />

  <div class="layout">
    <slot />
  </div>
{/if}

<style lang="postcss">
  .layout {
    @apply antialiased w-full flex flex-col;
  }
</style>
