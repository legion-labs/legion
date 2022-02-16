<script lang="ts">
  import { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
  import Home from "@/pages/Home.svelte";
  import authStatus from "@/stores/authStatus";

  export let initAuthStatus: InitAuthStatus | null;

  $: $authStatus = initAuthStatus;
</script>

<Home />
