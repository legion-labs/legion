<script lang="ts">
  import { onMount } from "svelte";

  import {
    GrpcWebImpl,
    HealthClientImpl,
  } from "@lgn/proto-telemetry/dist/health";
  import log from "@lgn/web-client/src/lib/log";

  import { getRemoteHost } from "@/lib/client";

  onMount(async () => {
    const client = new HealthClientImpl(
      new GrpcWebImpl("http://" + getRemoteHost() + ":9090", {})
    );

    const res = await client.check({ service: "analytics" });

    log.debug("health", res);
  });
</script>

<div>
  <h1>Health</h1>
</div>
