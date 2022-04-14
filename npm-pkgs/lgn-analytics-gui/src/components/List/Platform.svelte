<script lang="ts">
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  export let process: Process | undefined;

  function getPlatform(distro: string | undefined) {
    if (!distro) {
      return "Unknown";
    }
    const lowerCaseDistro = distro.toLowerCase();
    if (lowerCaseDistro.startsWith("windows")) {
      return "Windows";
    } else if (lowerCaseDistro.startsWith("ubuntu")) {
      return "Linux";
    }
    return "Unknown";
  }
</script>

<div class="flex gap-2 items-center">
  <i class="bi bi-pc text-content-38" />
  <span class="capitalize">{getPlatform(process?.distro)}</span>
</div>
