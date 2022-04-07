<script lang="ts">
  import { onMount } from "svelte";

  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";

  import Graph from "@/components/Graph/Graph.svelte";
  import { Route, Router } from "@/lib/navigator";
  import Health from "@/pages/Health.svelte";

  import ProcessList from "./components/List/ProcessList.svelte";
  import Log from "./components/Log/Log.svelte";
  import MetricsCanvas from "./components/Metric/MetricCanvas.svelte";
  import Header from "./components/Misc/Header.svelte";
  import TimelineRenderer from "./components/Timeline/Timeline.svelte";
  import LoadingBar from "./components/Misc/LoadingBar.svelte";

  export let initAuthStatus: InitAuthStatus | null;

  // TODO: Here we can control the UI and display a modal à la GitHub
  onMount(() => {
    if (initAuthStatus) {
      switch (initAuthStatus.type) {
        case "error": {
          window.location.href = initAuthStatus.authorizationUrl;
        }
      }
    }
  });
</script>

<svelte:head>
  <style>
    @import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap");
  </style>
</svelte:head>

<LoadingBar />
<div class="pt-2 pb-4 antialiased">
  <Header />
  <div class="pl-5 pr-5 pt-5 overflow-hidden">
    <Router>
      <Route path="/" primary={false}>
        <ProcessList />
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
        <Graph />
      </Route>
    </Router>
  </div>
</div>
