<script lang="ts">
  import { Router, Route } from "svelte-navigator";
  import Log from "@/pages/Log.svelte";
  import Timeline from "@/pages/Timeline.svelte";
  import Graph from "@/pages/Graph.svelte";
  import MetricsCanvas from "./components/MetricCanvas.svelte";
  import Header from "./components/Header.svelte";
  import ProcessList from "./components/ProcessList.svelte";
</script>

<div class="w-full h-screen p-2">
  <div class="grid">
    <Header />
    <div class="pl-5 pr-5 pb-5">
      <Router>
        <div id="app">
          <Route path="/" primary={false}>
            <ProcessList />
          </Route>
          <Route path="/log/:id" let:params let:location primary={false}>
            {#key params.id + location.search}
              <Log id={params.id} />
            {/key}
          </Route>
          <Route path="/timeline/:id" let:params let:location primary={false}>
            {#key params.id + location.search}
              <Timeline processId={params.id} />
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
    @apply text-center text-[#2c3e50];
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }
</style>
