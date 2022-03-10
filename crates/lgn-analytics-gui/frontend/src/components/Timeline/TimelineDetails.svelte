<script lang="ts">
  import { Process } from "@lgn/proto-telemetry/dist/process";
  export let process: Process;
</script>

{#if process}
  <div class="text-left text-sm">
    <span class="font-semibold inline-block">Process</span>
    <span>
      {process.exe.split("\\").pop()?.split("/").pop()}
    </span>
  </div>
{/if}
