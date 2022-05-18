<script lang="ts">
  import type { Process } from "@lgn/proto-telemetry/dist/process";

  import L10n from "../Misc/L10n.svelte";

  export let process: Process | undefined;

  function getPlatform(distro: string | undefined) {
    if (!distro) {
      return "unknown";
    }

    const lowerCaseDistro = distro.toLowerCase();

    if (lowerCaseDistro.startsWith("windows")) {
      return "windows";
    } else if (lowerCaseDistro.startsWith("ubuntu")) {
      return "linux";
    }

    return "unknown";
  }

  $: platform = getPlatform(process?.distro);
</script>

<div class="flex gap-2 items-center" title={platform}>
  <i class="bi bi-pc placeholder" />
  <span class="capitalize truncate">
    <L10n id="global-platform" variables={{ platform }} />
  </span>
</div>
