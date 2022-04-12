import { Level } from "@lgn/proto-log-stream/dist/log_stream";

export type Severity = "error" | "warn" | "info" | "trace" | "debug";

export function severityFromLevel(level: Level): Severity | null {
  switch (level) {
    case Level.DEBUG: {
      return "debug";
    }

    case Level.ERROR: {
      return "error";
    }

    case Level.INFO: {
      return "info";
    }

    case Level.TRACE: {
      return "trace";
    }

    case Level.WARN: {
      return "warn";
    }

    case Level.UNRECOGNIZED: {
      return null;
    }
  }
}

export type LogEntry = {
  id: number;
  message: string;
  severity: Severity;
  target: string;
  datetime: Date;
};
