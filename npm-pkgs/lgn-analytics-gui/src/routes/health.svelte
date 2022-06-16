<script lang="ts">
  import { onMount } from "svelte";
  import { getContext } from "svelte";

  import {
    GrpcWebImpl,
    HealthClientImpl,
  } from "@lgn/proto-telemetry/dist/health";
  import log from "@lgn/web-client/src/lib/log";

  const runtimeConfig = getContext("runtime-config");

  onMount(async () => {
    const client = new HealthClientImpl(
      new GrpcWebImpl("http://" + runtimeConfig.apiAnalytics.host + ":9090", {})
    );

    const res = await client.check({ service: "analytics" });

    log.debug("health", res);
  });
</script>

<div>
  <h1>Health</h1>
</div>
