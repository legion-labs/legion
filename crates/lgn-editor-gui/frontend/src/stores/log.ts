import { derived, get, writable } from "svelte/store";
import type { LogEntry } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";
import { initLogStream as initLogStreamApi } from "@/api";

export const buffer = 1_000;

export const streamedLogEntries = writable<Omit<LogEntry, "id">[]>([]);

export const logEntries = derived(
  streamedLogEntries,
  (streamedLogEntries) =>
    new Map(
      streamedLogEntries.map((log, index) => [index, { ...log, id: index }])
    )
);

export async function initLogStream() {
  const logStream = await initLogStreamApi();

  return logStream.subscribe((logEntry) => {
    if (get(streamedLogEntries).length > buffer - 1) {
      get(streamedLogEntries).shift();
    }

    // Defaulting to "trace" if the severity cannot be converted from the level
    const severity = severityFromLevel(logEntry.level) ?? "trace";

    streamedLogEntries.update((streamedLogEntries) => [
      ...streamedLogEntries,
      {
        severity,
        message: logEntry.message,
        target: logEntry.target,
        datetime: new Date(logEntry.time),
      },
    ]);
  });
}
