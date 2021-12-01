export function formatExecutionTime(time: number) {
  let unit = "ms";

  if (time < 1) {
    unit = "us";
    time = time * 1000;
    return time.toFixed(3) + " " + unit;
  }

  if (time > 1000) {
    unit = "seconds";
    time = time / 1000;
  }

  return time.toFixed(3) + " " + unit;
}
