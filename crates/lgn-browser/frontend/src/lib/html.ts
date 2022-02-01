/**
 * Takes a root element and a target element and automatically scrolls (y only)
 * to the target element if it becomes (even partly) invisible or hidden.
 *
 * If the `targetElement` is entirely visible in the `rootElement` nothing happens,
 * otherwise a scroll to the target element is performed.
 * @param rootElement the reference element that can be scrolled
 * @param targetElement the target element to keep visible
 */
export function keepElementVisible(
  rootElement: Element,
  targetElement: Element
) {
  const targetElementRect = targetElement.getBoundingClientRect();
  const rootElementReact = rootElement.getBoundingClientRect();

  if (
    !(
      targetElementRect.top - rootElementReact.top >= 0 &&
      targetElementRect.bottom < rootElementReact.bottom
    )
  ) {
    rootElement.scroll({
      top:
        rootElement.scrollTop + (targetElementRect.top - rootElementReact.top),
      left: rootElement.scrollLeft,
      behavior: "auto",
    });
  }
}

/**
 * Converts x rem to pixels. Uses the actual `fontSize` for accuracy.
 *
 * Returns `null` if `rem` is lt 0 or if the `fontSize` can't be read.
 */
export function remToPx(rem: number): number | null {
  if (rem < 0) {
    return null;
  }

  if (rem === 0) {
    return 0;
  }

  const fontSizeInPx = getComputedStyle(document.documentElement).fontSize;

  const parsedFontSize = fontSizeInPx.match(/(\d+)px/)?.[1];

  if (!parsedFontSize) {
    return null;
  }

  const fontSize = +parsedFontSize;

  if (isNaN(fontSize)) {
    return null;
  }

  return rem * fontSize;
}
