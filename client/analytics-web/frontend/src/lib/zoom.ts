export function zoomHorizontalViewRange(
  currentViewRange: [number, number],
  windowWidthPx: number,
  event: WheelEvent
): [number, number] {
  const speed = 0.75;
  const factor = event.deltaY > 0 ? 1.0 / speed : speed;
  const length = currentViewRange[1] - currentViewRange[0];
  const newLength = length * factor;
  const pctCursor = event.offsetX / windowWidthPx;
  const pivot = currentViewRange[0] + length * pctCursor;
  return [pivot - newLength * pctCursor, pivot + newLength * (1 - pctCursor)];
}
