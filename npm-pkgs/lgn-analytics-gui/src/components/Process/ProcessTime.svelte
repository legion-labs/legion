<script lang="ts">
  import { formatDistance } from "date-fns";

  import type { Process } from "@lgn/proto-telemetry/dist/process";

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
  <i class="bi bi-clock-fill text-placeholder mr-1" />
  {formatLocalTime(process?.startTime ?? "")}
  <span class="text-xs">
    ({formatDistance(new Date(process?.startTime ?? ""), new Date(), {
      addSuffix: true,
    })})
  </span>
</div>
