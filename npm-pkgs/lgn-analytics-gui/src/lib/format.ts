import type { Process } from "@lgn/proto-telemetry/dist/process";

/**
 * Formats the provided time (in ms) "Hours:Minutes:Seconds.SecondsFraction".
 *
 * ## Examples
 *
 * ```ts
 * formatTime(10) === "00:00:00.010";
 * formatTime(1_000) === "00:00:01.000";
 * formatTime(1_000_000_000) === "277:16666:40.000";
 * ```
 */
export function formatTime(ms: number) {
  const seconds = ms / 1_000;
  const secondsWhole = Math.floor(seconds);
  const secondsStr = String(secondsWhole % 60).padStart(2, "0");
  const secondsFraction = String(Math.round(ms % 1_000)).padStart(3, "0");
  const minutes = secondsWhole / 60;
  const minutesWhole = Math.floor(minutes);
  const minutesStr = String(minutesWhole).padStart(2, "0");
  const hours = minutesWhole / 60;
  const hoursWhole = Math.floor(hours);
  const hoursStr = String(hoursWhole).padStart(2, "0");

  return hoursStr + ":" + minutesStr + ":" + secondsStr + "." + secondsFraction;
}

/**
 * Formats the provided time (in ms) and an optional fraction digits (defaults to `3`)
 * to a string of format "VALUE unit" _or_ "Hours:Minutes:Seconds" if the time is too large.
 *
 * ## Examples
 *
 * ```ts
 * formatExecutionTime(10) === "10.000 ms";
 * formatExecutionTime(1_000) === "1000.000 ms";
 * formatExecutionTime(1_000, 5) === "1000.00000 ms";
 * formatExecutionTime(1_000_000_000) === "277:46:40";
 * ```
 */
export function formatExecutionTime(timeMs: number, fractionDigits = 3) {
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
    unit = "s";
    time = time / 1000;
    if (time > 60) {
      time = Math.round(time);
      const secondsWhole = String(time % 60).padStart(2, "0");
      const minutes = String(Math.floor((time / 60) % 60)).padStart(2, "0");
      const hours = String(Math.floor(time / (60 * 60))).padStart(2, "0");
      return `${sign}${hours}:${minutes}:${secondsWhole}`;
    }
  }
  return sign + time.toFixed(fractionDigits) + " " + unit;
}

export function formatProcessName(process: Process) {
  return process.exe.split("/").pop()?.split("\\").pop();
}
