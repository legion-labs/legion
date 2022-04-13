import { derived, get, writable } from "svelte/store";

import { throttled } from "@lgn/web-client/src/lib/store";
import type { LogEntry } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";

import { initEditorLogStream, initRuntimeLogStream } from "@/api";

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

export function initLogStreams() {
  const editorSubscription = initEditorLogStream().subscribe(
    ({ lagging, traceEvent }) => {
      if (get(streamedLogEntries).length > buffer - 1) {
        get(streamedLogEntries).shift();
      }

      if (typeof lagging === "number") {
        // TODO: Handle lagging messages

        return;
      }

      if (traceEvent) {
        // Defaulting to "trace" if the severity cannot be converted from the level
        const severity = severityFromLevel(traceEvent.level) ?? "trace";

        streamedLogEntries.update((streamedLogEntries) => [
          ...streamedLogEntries,
          {
            severity,
            message: traceEvent.message,
            source: "editor",
            target: traceEvent.target,
            datetime: new Date(traceEvent.time),
          },
        ]);
      }
    }
  );

  const runtimeSubscription = initRuntimeLogStream().subscribe(
    ({ lagging, traceEvent }) => {
      if (get(streamedLogEntries).length > buffer - 1) {
        get(streamedLogEntries).shift();
      }

      if (typeof lagging === "number") {
        // TODO: Handle lagging messages

        return;
      }

      if (traceEvent) {
        // Defaulting to "trace" if the severity cannot be converted from the level
        const severity = severityFromLevel(traceEvent.level) ?? "trace";

        streamedLogEntries.update((streamedLogEntries) => [
          ...streamedLogEntries,
          {
            severity,
            message: traceEvent.message,
            source: "runtime",
            target: traceEvent.target,
            datetime: new Date(traceEvent.time),
          },
        ]);
      }
    }
  );

  return () => {
    editorSubscription.unsubscribe();
    runtimeSubscription.unsubscribe();
  };
}
