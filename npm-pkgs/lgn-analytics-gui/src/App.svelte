<script lang="ts">
  import { onMount, setContext } from "svelte";

  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import { replaceClassesWith } from "@lgn/web-client/src/lib/html";
  import { createThemeStore } from "@lgn/web-client/src/stores/theme";

  import CallGraphFlat from "@/components/CallGraphFlat/CallGraphFlat.svelte";
  import { Route, Router } from "@/lib/navigator";
  import Health from "@/pages/Health.svelte";

  import Log from "./components/Log/Log.svelte";
  import MetricsCanvas from "./components/Metric/MetricCanvas.svelte";
  import Header from "./components/Misc/Header.svelte";
  import LoadingBar from "./components/Misc/LoadingBar.svelte";
  import ProcessPage from "./components/Process/ProcessPage.svelte";
  import TimelineRenderer from "./components/Timeline/Timeline.svelte";
  import { themeContextKey, themeStorageKey } from "./constants";

  export let initAuthStatus: InitAuthStatus | null;

  const theme = createThemeStore(themeStorageKey, "dark");

  setContext(themeContextKey, theme);

  // TODO: Here we can control the UI and display a modal like in the Editor
  onMount(() => {
    const unsubscribe = theme.subscribe(({ name }) => {
      replaceClassesWith(document.body, `theme-${name}`);
    });

    if (initAuthStatus) {
      switch (initAuthStatus.type) {
        case "error": {
          window.location.href = initAuthStatus.authorizationUrl;
        }
      }
    }

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
