export type LogLevel = "debug" | "error" | "info" | "trace" | "warn";

export type Log = {
  id: number;
  message: string;
  severity: LogLevel;
  target: string;
  timestamp: Date;
};
