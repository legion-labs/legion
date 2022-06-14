import type { Log } from "@lgn/apis/log";

export type Severity = "error" | "warn" | "info" | "trace" | "debug";

export function severityFromLevel(level: Log.TraceEventLevel): Severity | null {
  switch (level) {
    case "Debug": {
      return "debug";
    }

    case "Error": {
      return "error";
    }

    case "Info": {
      return "info";
    }

    case "Trace": {
      return "trace";
    }

    case "Warn": {
      return "warn";
    }

    default: {
      return null;
    }
  }
}

export type Source = "editor" | "runtime";

export type LogEntry = {
  id: number;
  message: string;
  severity: Severity;
  source: Source;
  target: string;
  datetime: Date;
};
