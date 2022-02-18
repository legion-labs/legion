<script lang="ts">
  import { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import { onMount } from "svelte";
  import Header from "./components/Misc/Header.svelte";

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

      <!--      <Route path="/" primary={false}>
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
          </Route> -->
      <!-- </Router> --> 


<div class="w-full h-screen p-2">
  <div class="grid">
    <Header />
    <div class="p-5">
       <div id="app">
         <slot />
    </div>
</div>
</div>
</div>
