<script lang="ts">
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  export let process: Process | undefined;

  function getComputer() {
    if (!process) {
      return "Unknown";
    }
    if (!process.computer) {
      return "Unknown";
    }
    return process.computer;
  }
</script>

<div
  title={[process?.distro, process?.cpuBrand].join("\n")}
  class="flex gap-2 items-center"
>
  <i class="bi bi-pc text-placeholder" />
  <span>{getComputer()}</span>
</div>
