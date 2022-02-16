<script lang="ts">
  import { onMount } from "svelte";
  import { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import Home from "@/pages/Home.svelte";

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

<Home />
