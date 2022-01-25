// Taken from https://svelte.dev/repl/0ace7a508bd843b798ae599940a91783?version=3.16.7
/**
 * When the user clicks outside the provided `node` (that is, on an element on the page
 * that isn't contained by the `node`) the `listener` function is called.
 *
 * To click inside the context menu (if it's present in the page) will _not_ trigger
 * the click outside function.
 *
 * @param node The `node`
 * @param listener The function called when the user clicks outside the `ref node`
 */
export default function clickOutside(
  node: Node,
  listener: (event: MouseEvent) => void
) {
  const handleClick = (event: MouseEvent) => {
    const contextMenu = document.getElementById("context-menu");

    if (
      // Target is not a valid Node
      !(event.target instanceof Node) ||
      // The context menu is in the page and contains the event target
      (contextMenu && contextMenu.contains(event.target)) ||
      // The node contains the event target
      node.contains(event.target) ||
      // The event has been "prevented"
      event.defaultPrevented
    ) {
      return;
    }

    listener(event);
  };

  window.addEventListener("click", handleClick, true);

  return {
    destroy() {
      window.removeEventListener("click", handleClick, true);
    },
  };
}
