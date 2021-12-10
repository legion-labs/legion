export type SelectionState = {
  beginMouseX: number | undefined;
  selectedRange: [number, number] | undefined;
};

export function NewSelectionState(): SelectionState {
  return { beginMouseX: undefined, selectedRange: undefined };
}

export function DrawSelectedRange(
  canvas: HTMLCanvasElement,
  renderingContext: CanvasRenderingContext2D,
  selectionState: SelectionState,
  viewRange: [number, number]
) {
  if (!selectionState.selectedRange) {
    return;
  }

  const [begin, end] = viewRange;
  const invTimeSpan = 1.0 / (end - begin);
  const canvasWidth = canvas.clientWidth;
  const canvasHeight = canvas.clientHeight;
  const msToPixelsFactor = invTimeSpan * canvasWidth;
  const [beginSelection, endSelection] = selectionState.selectedRange;
  const beginPixels = (beginSelection - begin) * msToPixelsFactor;
  const endPixels = (endSelection - begin) * msToPixelsFactor;

  renderingContext.fillStyle = "rgba(64, 64, 200, 0.2)";
  renderingContext.fillRect(
    beginPixels,
    0,
    endPixels - beginPixels,
    canvasHeight
  );
}

function UpdateSelectedRange(
  event: MouseEvent,
  windowWidthPx: number,
  currentViewRange: [number, number],
  selectionState: SelectionState
) {
  if (!selectionState.beginMouseX) {
    selectionState.beginMouseX = event.offsetX;
  }

  const factor = (currentViewRange[1] - currentViewRange[0]) / windowWidthPx;
  const beginTime = currentViewRange[0] + factor * selectionState.beginMouseX;
  const endTime = currentViewRange[0] + factor * event.offsetX;
  selectionState.selectedRange = [beginTime, endTime];
}

// RangeSelectionOnMouseMove returns the selected range has been updated
// and therefore its visual representation needs to be refreshed
export function RangeSelectionOnMouseMove(
  event: MouseEvent,
  selectionState: SelectionState,
  canvas: HTMLCanvasElement,
  currentViewRange: [number, number]
): boolean {
  if (event.buttons !== 1) {
    selectionState.beginMouseX = undefined;
    return false;
  }

  if (event.shiftKey) {
    UpdateSelectedRange(event, canvas.width, currentViewRange, selectionState);
    return true;
  }
  return false;
}

// RangeSelectionOnMouseDown returns the selected range has been updated
// and therefore its visual representation needs to be refreshed
export function RangeSelectionOnMouseDown(
  event: MouseEvent,
  selectionState: SelectionState
): boolean {
  if (
    event.shiftKey &&
    (selectionState.beginMouseX || selectionState.selectedRange)
  ) {
    selectionState.beginMouseX = undefined;
    selectionState.selectedRange = undefined;
    return true;
  }
  return false;
}
