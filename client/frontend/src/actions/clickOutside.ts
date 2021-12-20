// Taken from https://svelte.dev/repl/0ace7a508bd843b798ae599940a91783?version=3.16.7
/**
 * When the user clicks outside the provided `node` (that is, on an element on the page
 * that isn't contained by the `node`) the `listener` function is callled.
 * @param node The `node`
 * @param listener The function called when the user clicks outside the `ref node`
 */
export default function clickOutside(
  node: Node,
  listener: (event: MouseEvent) => void
) {
  const handleClick = (event: MouseEvent) => {
    if (
      node &&
      event.target instanceof Node &&
      !node.contains(event.target) &&
      !event.defaultPrevented
    ) {
      listener(event);
    }
  };

  window.addEventListener("click", handleClick, true);

  return {
    destroy() {
      window.removeEventListener("click", handleClick, true);
    },
  };
}
