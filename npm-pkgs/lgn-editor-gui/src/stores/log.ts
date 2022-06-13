import { derived, get, writable } from "svelte/store";

import type { Log } from "@lgn/apis/log";
import { displayError } from "@lgn/web-client/src/lib/errors";
import log from "@lgn/web-client/src/lib/log";
import { throttled } from "@lgn/web-client/src/lib/store";
import type { LogEntry, Source } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";

import { getEditorTraceEvents, getRuntimeTraceEvents } from "@/api";

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
  const processLogStreamResponse = (
    traceEvents: Log.TraceEvent[],
    source: Source
  ) => {
    if (get(streamedLogEntries).length > buffer - 1) {
      get(streamedLogEntries).shift();
    }

    if (traceEvents.length) {
      for (const traceEvent of traceEvents) {
        // Defaulting to "trace" if the severity cannot be converted from the level
        const severity = severityFromLevel(traceEvent.level) ?? "trace";

        streamedLogEntries.update((streamedLogEntries) => [
          ...streamedLogEntries,
          {
            severity,
            message: traceEvent.message,
            source,
            target: traceEvent.target,
            datetime: new Date(Number(traceEvent.time)),
          },
        ]);
      }
    }
  };

  const editorInterval = setInterval(() => {
    getEditorTraceEvents()
      .then((traceEventsResponse) => {
        processLogStreamResponse(traceEventsResponse.value, "editor");
      })
      .catch((error) => {
        log.error("log::editor", `an error occured: ${displayError(error)}`);
      });
  }, 2_000);

  const runtimeInterval = setInterval(() => {
    getRuntimeTraceEvents()
      .then((traceEventsResponse) => {
        processLogStreamResponse(traceEventsResponse.value, "runtime");
      })
      .catch((error) => {
        log.error("log::runtime", `an error occured: ${displayError(error)}`);
      });
  }, 2_000);

  return () => {
    clearInterval(editorInterval);
    clearInterval(runtimeInterval);
  };
}
