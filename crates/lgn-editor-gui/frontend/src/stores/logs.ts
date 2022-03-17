import { derived, get, writable } from "svelte/store";
import type { Log } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";
import { streamLogs } from "@/api";

export const buffer = 1_000;

export const streamedLogs = writable<Omit<Log, "id">[]>([]);

export const logs = derived(
  streamedLogs,
  (streamedLogs) =>
    new Map(streamedLogs.map((log, index) => [index, { ...log, id: index }]))
);

export async function initStreamLogs() {
  const logs = await streamLogs();

  return logs.subscribe((log) => {
    if (get(streamedLogs).length > buffer - 1) {
      get(streamedLogs).shift();
    }

    // Defaulting to "trace" if the severity cannot be converted from the level
    const severity = severityFromLevel(log.level) ?? "trace";

    streamedLogs.update((streamedLogs) => [
      ...streamedLogs,
      {
        severity,
        message: log.message,
        target: log.target,
        datetime: new Date(log.time),
      },
    ]);
  });
}
