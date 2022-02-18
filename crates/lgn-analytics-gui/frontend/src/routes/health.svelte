<script lang="ts">
  import {
    GrpcWebImpl,
    HealthClientImpl,
  } from "@lgn/proto-telemetry/dist/health";
  import { onMount } from "svelte";
  import { getRemoteHost } from "@/lib/client";

  onMount(async () => {
    const client = new HealthClientImpl(
      new GrpcWebImpl("http://" + getRemoteHost() + ":9090", {})
    );
    const res = await client.check({ service: "analytics" });
    console.log(res);
  });
</script>

<div>
  <h1>Health</h1>
</div>
