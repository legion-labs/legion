export function formatExecutionTime(time: number) {
  let unit = "ms";

  if (Math.abs(time) < 1) {
    unit = "us";
    time = time * 1000;
    return time.toFixed(3) + " " + unit;
  }

  if (Math.abs(time) > 1000) {
    unit = "seconds";
    time = time / 1000;
  }

  return time.toFixed(3) + " " + unit;
}
