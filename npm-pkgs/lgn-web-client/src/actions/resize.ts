// TODO: We can probably get rid of this action at one point
/**
 * Takes an `element` and a `listener` and calls the `listener` each time the `element`'s size changes.
 *
 * You can use this action as an alternative to [dimensions binding](https://svelte.dev/tutorial/dimensions)
 * except the provided function is called _once_ per resize, i.e. when both the height and the width changed.
 * It also uses the "event listener" approach vs the reactive one.
 *
 * _Consider using the Svelte [dimensions binding](https://svelte.dev/tutorial/dimensions)
 * before using this action._
 * @param element The `element` to watch resize events from
 * @param listener Function called with the new `DOMRectReadOnly` properties of the watched `element`
 */
export default function resize(
  element: Element,
  listener: (rect: DOMRectReadOnly) => void
) {
  const observer = new ResizeObserver(([entry]) => {
    listener(entry.contentRect);
  });

  observer.observe(element);

  return {
    destroy() {
      observer.disconnect();
    },
  };
}
