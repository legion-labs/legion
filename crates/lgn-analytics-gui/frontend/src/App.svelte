<script lang="ts">
  import { Router, links, Route, globalHistory } from "svelte-navigator";
  import Home from "@/pages/Home.svelte";
  import About from "@/pages/About.svelte";
  import Log from "@/pages/Log.svelte";
  import Timeline from "@/pages/Timeline.svelte";
  import Graph from "@/pages/Graph.svelte";
  import MetricsCanvas from "./components/MetricCanvas.svelte";
  import { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import { onMount } from "svelte";

  const historyStore = { subscribe: globalHistory.listen };

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

<Router>
  <div id="app">
    <div id="nav" use:links>
      <a
        href="/"
        class:router-link-exact-active={$historyStore.location.pathname === "/"}
      >
        Home
      </a>
      |
      <a
        href="/about"
        class:router-link-exact-active={$historyStore.location.pathname ===
          "/about"}
      >
        About
      </a>
    </div>
    <Route path="/" primary={false}><Home /></Route>
    <Route path="/about"><About /></Route>
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
    <Route path="/cumulative-call-graph" primary={false}><Graph /></Route>
  </div>
</Router>

<style lang="postcss">
  #app {
    @apply text-center text-[#2c3e50];
    font-family: Avenir, Helvetica, Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  #nav {
    @apply p-8;
  }

  #nav a {
    @apply font-bold text-[#2c3e50] underline;
  }

  #nav a.router-link-exact-active {
    @apply font-bold text-[#ca2f0f] underline;
  }
</style>
