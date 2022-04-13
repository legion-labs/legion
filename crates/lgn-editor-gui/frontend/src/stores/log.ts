import { derived, get, writable } from "svelte/store";

import { throttled } from "@lgn/web-client/src/lib/store";
import type { LogEntry } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";

import { initEditorLogStream, initRuntimeLogStream } from "@/api";
import type { InitLogStreamResponse } from "@lgn/proto-log-stream/dist/log_stream";

export const buffer = 1_000;

export const streamedLogEntries = writable<Omit<LogEntry, "id">[]>([]);

export const logEntries = throttled(
  derived(
    streamedLogEntries,
    (streamedLogEntries) =>
      new Map(
        streamedLogEntries.map((log, index) => [index, { ...log, id: index }])
      )
  ),
  500
);

function processLog(logEntry: InitLogStreamResponse) {
  if (get(streamedLogEntries).length > buffer - 1) {
    get(streamedLogEntries).shift();
  }

  if (typeof logEntry.lagging === "number") {
    // TODO: Handle lagging messages

    return;
  }

  if (logEntry.traceEvent) {
    // Defaulting to "trace" if the severity cannot be converted from the level
    const severity = severityFromLevel(logEntry.traceEvent.level) ?? "trace";

    streamedLogEntries.update((streamedLogEntries) => [
      ...streamedLogEntries,
      {
        severity,
        message: logEntry.traceEvent.message,
        target: logEntry.traceEvent.target,
        datetime: new Date(logEntry.traceEvent.time),
      },
    ]);
  }
}

export function initLogStreams() {
  const editorSubscription = initEditorLogStream().subscribe(processLog);
  const runtimeSubscription = initRuntimeLogStream().subscribe(processLog);

  return () => {
    editorSubscription.unsubscribe();
    runtimeSubscription.unsubscribe();
  };
}
