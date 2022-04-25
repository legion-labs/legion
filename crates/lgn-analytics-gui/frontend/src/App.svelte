<script lang="ts">
  import { onMount } from "svelte";

  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";

  import Graph from "@/components/Graph/Graph.svelte";
  import { Route, Router } from "@/lib/navigator";
  import Health from "@/pages/Health.svelte";
  import Log from "@/pages/Log.svelte";

  import ProcessList from "./components/List/ProcessList.svelte";
  import MetricsCanvas from "./components/Metric/MetricCanvas.svelte";
  import Header from "./components/Misc/Header.svelte";
  import LoadingBar from "./components/Misc/LoadingBar.svelte";
  import TimelineRenderer from "./components/Timeline/Timeline.svelte";

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

<div class="w-full h-screen p-0">
  <LoadingBar />
  <div class="grid">
    <Header />
    <div class="pl-0 pr-0 pt-0 overflow-hidden">
      <Router>
        <div id="app">
          <Route path="/" primary={false}>
            <ProcessList />
          </Route>
          <Route path="/health"><Health /></Route>
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
        </div>
      </Router>
    </div>
  </div>
</div>

<style lang="postcss">
  #app {
    @apply text-center text-[#000000e0];
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }
  .grid {
    background-color: #f6f6f6;
  }
</style>
