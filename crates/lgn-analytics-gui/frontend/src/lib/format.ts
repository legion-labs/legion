import { Process } from "@lgn/proto-telemetry/dist/process";

export function formatExecutionTime(timeMs: number) {
  if (!isFinite(timeMs)) {
    return "";
  }
  let unit = "ns";
  let time = timeMs * 1000000; //If there are problems of numeric stability we could test early for cases >= 1 minute

  let sign = "";
  if (time < 0) {
    time = Math.abs(time);
    sign = "-";
  }

  if (time > 1000) {
    unit = "us";
    time = time / 1000;
  }
  if (time > 1000) {
    unit = "ms";
    time = time / 1000;
  }
  if (time > 1000) {
    unit = "seconds";
    time = time / 1000;
    if (time > 60) {
      time = Math.round(time);
      const secondsWhole = String(time % 60).padStart(2, "0");
      const minutes = String(Math.floor((time / 60) % 60)).padStart(2, "0");
      const hours = String(Math.floor(time / (60 * 60))).padStart(2, "0");
      return `${sign}${hours}:${minutes}:${secondsWhole}`;
    }
  }
  return sign + time.toFixed(3) + " " + unit;
}

export function formatProcessName(process: Process) {
  return process.exe.split("/").pop()?.split("\\").pop();
}
