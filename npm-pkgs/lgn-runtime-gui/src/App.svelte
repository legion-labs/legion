<script lang="ts">
  import { onMount } from "svelte";

  import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import log from "@lgn/web-client/src/lib/log";

  import Home from "@/pages/Home.svelte";

  export let initAuthStatus: InitAuthStatus | null;

  export let dispose: () => void | undefined;

  // TODO: Here we can control the UI and display a modal à la GitHub
  onMount(() => {
    if (initAuthStatus) {
      switch (initAuthStatus.type) {
        case "error": {
          if (initAuthStatus.authorizationUrl !== null) {
            window.location.href = initAuthStatus.authorizationUrl;
          } else {
            log.warn("auth", "User is not authed");
          }
        }
      }
    }

    return () => {
      dispose?.();
    };
  });
</script>

<Home />
