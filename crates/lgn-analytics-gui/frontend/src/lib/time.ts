import { Process } from "@lgn/proto-telemetry/dist/process";

export function timestampToMs(process: Process, timestamp: number): number {
  const nbTicks = timestamp - process.startTicks;
  return (nbTicks * 1000.0) / process.tscFrequency;
}

export function processMsOffsetToRoot(
  currentProcess: Process | undefined,
  process: Process
): number {
  if (!currentProcess?.startTime) {
    throw new Error("Parent process start time undefined");
  }
  const parentStartTime = Date.parse(currentProcess?.startTime);
  return Date.parse(process.startTime) - parentStartTime;
}
