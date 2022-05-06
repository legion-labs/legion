<script lang="ts">
  import { onMount, setContext } from "svelte";

  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import { displayError } from "@lgn/web-client/src/lib/errors";
  import { replaceClassesWith } from "@lgn/web-client/src/lib/html";
  import log from "@lgn/web-client/src/lib/log";
  import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
  import { createL10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";
  import { createThemeStore } from "@lgn/web-client/src/stores/theme";

  import en from "@/assets/locales/en-US/example.ftl?raw";
  import fr from "@/assets/locales/fr-CA/example.ftl?raw";
  import CallGraphFlat from "@/components/CallGraphFlat/CallGraphFlat.svelte";
  import { Route, Router } from "@/lib/navigator";
  import Health from "@/pages/Health.svelte";

  import Log from "./components/Log/Log.svelte";
  import MetricsCanvas from "./components/Metric/MetricCanvas.svelte";
  import Header from "./components/Misc/Header.svelte";
  import LoadingBar from "./components/Misc/LoadingBar.svelte";
  import ProcessPage from "./components/Process/ProcessPage.svelte";
  import TimelineRenderer from "./components/Timeline/Timeline.svelte";
  import { getThreadItemLength } from "./components/Timeline/Values/TimelineValues";
  import {
    httpClientContextKey,
    l10nOrchestratorContextKey,
    localeStorageKey,
    themeContextKey,
    themeStorageKey,
    threadItemLengthContextKey,
    threadItemLengthFallback,
  } from "./constants";
  import { makeGrpcClient } from "./lib/client";

  export let initAuthStatus: InitAuthStatus | null;

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
        connect: {
          key: localeStorageKey,
          storage: new DefaultLocalStorage(),
        },
      },
    }
  );

  setContext(themeContextKey, theme);

  setContext(l10nOrchestratorContextKey, l10n);

  setContext(httpClientContextKey, makeGrpcClient());

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

    return unsubscribe;
  });
</script>

<LoadingBar />

<div class="pt-2 pb-4 antialiased">
  <Header />
  <div class="pl-5 pr-5 pt-5 overflow-hidden">
    <Router>
      <Route path="/" primary={false}>
        <ProcessPage />
      </Route>
      <Route path="/health">
        <Health />
      </Route>
      <Route path="/log/:id" let:params let:location primary={false}>
        {#key params.id + location.search}
          <Log id={params.id} />
        {/key}
      </Route>
      <Route path="/timeline/:id" let:params let:location primary={false}>
        {#key params.id + location.search}
          <TimelineRenderer processId={params.id} />
        {/key}
      </Route>
      <Route path="/metrics/:id" let:params primary={false}>
        {#key params.id}
          <MetricsCanvas id={params.id} />
        {/key}
      </Route>
      <Route path="/cumulative-call-graph" primary={false}>
        <CallGraphFlat />
      </Route>
    </Router>
  </div>
</div>
