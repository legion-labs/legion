<script lang="ts">
  import { Router, links, Route, globalHistory } from "svelte-navigator";
  import Home from "@/pages/Home.svelte";
  import About from "@/pages/About.svelte";
  import Log from "@/pages/Log.svelte";
  import Timeline from "@/pages/Timeline.svelte";
  import Graph from "@/pages/Graph.svelte";

  const historyStore = { subscribe: globalHistory.listen };
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
    <Route path="/"><Home /></Route>
    <Route path="/about"><About /></Route>
    <Route path="/log/:id" let:params primary={false}><Log id={params.id} /></Route>
    <Route path="/timeline/:id" let:params primary={false}>
      {#key params.id}
        <Timeline id={params.id} />
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
    @apply font-bold text-[#42b983] underline;
  }
</style>
