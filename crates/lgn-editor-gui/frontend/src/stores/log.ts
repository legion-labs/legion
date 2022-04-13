import { derived, get, writable } from "svelte/store";

import { throttled } from "@lgn/web-client/src/lib/store";
import type { LogEntry, Source } from "@lgn/web-client/src/types/log";
import { severityFromLevel } from "@lgn/web-client/src/types/log";
import type { TraceEvent } from "@lgn/proto-log-stream/dist/log_stream";

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
  const processLogStreamResponse = (
    lagging: number | undefined,
    traceEvent: TraceEvent | undefined,
    source: Source
  ) => {
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
          source,
          target: traceEvent.target,
          datetime: new Date(traceEvent.time),
        },
      ]);
    }
  };

  const editorSubscription = initEditorLogStream().subscribe(
    ({ lagging, traceEvent }) => {
      processLogStreamResponse(lagging, traceEvent, "editor");
    }
  );
  const runtimeSubscription = initRuntimeLogStream().subscribe(
    ({ lagging, traceEvent }) => {
      processLogStreamResponse(lagging, traceEvent, "runtime");
    }
  );

  return () => {
    editorSubscription.unsubscribe();
    runtimeSubscription.unsubscribe();
  };
}
