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
  rootElement: HTMLElement,
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
