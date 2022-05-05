<script lang="ts">
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  export let process: Process | undefined;

  $: computer = process?.computer || "Unknown";
</script>

<div
  title={[process?.distro, process?.cpuBrand].join("\n")}
  class="flex gap-2 items-center"
>
  <i class="bi bi-pc placeholder" />
  <span>
    <slot {computer}>
      {computer}
    </slot>
  </span>
</div>
