<script lang="ts">
  import { Process } from "@lgn/proto-telemetry/dist/process";
  import { formatDistance } from "date-fns";
  export let process: Process | undefined;

  function formatLocalTime(timeStr: string): string {
    const time = new Date(timeStr);
    return time.toLocaleTimeString(navigator.language, {
      timeZoneName: "short",
      hour12: false,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
  }
</script>

<div>
  <i class="bi bi-clock-fill text-neutral-400" />
  {formatLocalTime(process?.startTime ?? "")}
  <span class="text-xs">
    ({formatDistance(new Date(process?.startTime ?? ""), new Date(), {
      addSuffix: true,
    })})
  </span>
</div>
