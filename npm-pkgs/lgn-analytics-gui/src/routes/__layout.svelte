<script lang="ts" context="module">
  import { goto } from "$app/navigation";
  import type { Load } from "@sveltejs/kit";

  import { headlessRun } from "@lgn/web-client";
  import type { Level } from "@lgn/web-client/src/lib/log";
  import {
    ConsoleTransport,
    NotificationsTransport,
  } from "@lgn/web-client/src/lib/log/transports";
  import { createNotificationsStore } from "@lgn/web-client/src/stores/notifications";

  const redirectUri = document.location.origin + "/";

  const notifications = createNotificationsStore<Fluent>();

  export const load: Load = async ({ fetch, url }) => {
    try {
      const { dispose, initAuthStatus } = await headlessRun({
        auth: {
          fetch,
          issuerUrl:
            "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz",
          redirectUri,
          clientId: "2kp01gr54dfc7qp1325hibcro3",
          login: {
            cookies: {
              accessToken: "analytics_access_token_v2",
              refreshToken: "analytics_refresh_token_v2",
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

      return { props: { dispose, initAuthStatus, notifications } };
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
  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import { displayError } from "@lgn/web-client/src/lib/errors";
  import { replaceClassesWith } from "@lgn/web-client/src/lib/html";
  import log from "@lgn/web-client/src/lib/log";
  import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
  import { createL10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";
  import accessToken from "@lgn/web-client/src/stores/accessToken";
  import type { NotificationsStore } from "@lgn/web-client/src/stores/notifications";
  import { createThemeStore } from "@lgn/web-client/src/stores/theme";

  import en from "@/assets/locales/en-US/main.ftl?raw";
  import fr from "@/assets/locales/fr-CA/main.ftl?raw";
  import Header from "@/components/Misc/Header.svelte";
  import LoadingBar from "@/components/Misc/LoadingBar.svelte";
  import { getThreadItemLength } from "@/components/Timeline/Values/TimelineValues";
  import {
    debugContextKey,
    httpClientContextKey,
    l10nOrchestratorContextKey,
    localeStorageKey,
    notificationsContextKey,
    themeContextKey,
    themeStorageKey,
    threadItemLengthContextKey,
    threadItemLengthFallback,
  } from "@/constants";
  import { makeGrpcClient } from "@/lib/client";

  import "../assets/index.css";

  export let initAuthStatus: InitAuthStatus | null;

  export let notifications: NotificationsStore<Fluent>;

  export let dispose: () => void | undefined;

  const theme = createThemeStore(themeStorageKey, "dark");

  const l10n = createL10nOrchestrator(
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

  setContext(themeContextKey, theme);

  setContext(l10nOrchestratorContextKey, l10n);

  setContext(httpClientContextKey, makeGrpcClient($accessToken));

  setContext(notificationsContextKey, notifications);

  setContext(
    debugContextKey,
    writable(import.meta.env.VITE_LEGION_ANALYTICS_DEBUG === "true")
  );

  try {
    setContext(threadItemLengthContextKey, getThreadItemLength());
  } catch (error) {
    log.warn(
      `Couldn't get the proper thread item length, defaulting to the arbitrary value "${threadItemLengthFallback}": ${displayError(
        error
      )}`
    );

    setContext(threadItemLengthContextKey, threadItemLengthFallback);
  }

  // TODO: Here we can control the UI and display a modal like in the Editor
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

<Notifications store={notifications} />

<LoadingBar />

<Header />

<div class="pt-2 pb-4 antialiased">
  <div class="pl-5 pr-5 pt-5 overflow-hidden">
    <slot />
  </div>
</div>
