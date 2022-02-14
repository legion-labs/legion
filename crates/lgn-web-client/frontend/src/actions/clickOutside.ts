// Taken from https://svelte.dev/repl/0ace7a508bd843b798ae599940a91783?version=3.16.7
/**
 * When the user clicks outside the provided `node` (that is, on an element on the page
 * that isn't contained by the `node`) the `listener` function is called.
 *
 * Additionally, "ignored" nodes can be provided with the `ignoredNodes` attribute.
 * When the user clicks on any of these nodes, the `listener` is not called.
 */
export default function clickOutside(node: Node, ignoredNodes: Node[] = []) {
  const handleMouseUp = (event: MouseEvent) => {
    if (
      // Target is not a valid Node
      !(event.target instanceof Node) ||
      // Any of the ignored nodes contains the event target
      ignoredNodes.some((ignoredNode) =>
        ignoredNode.contains(event.target as Node)
      ) ||
      // The node contains the event target
      node.contains(event.target) ||
      // The event has been "prevented"
      event.defaultPrevented
    ) {
      return;
    }

    node.dispatchEvent(
      new CustomEvent("click-outside", {
        detail: { originalEvent: event },
      })
    );
  };

  window.addEventListener("mouseup", handleMouseUp);

  return {
    update(newlyIgnoredNodes: Node[]) {
      if (newlyIgnoredNodes) {
        ignoredNodes = newlyIgnoredNodes;
      }
    },
    destroy() {
      window.removeEventListener("mouseup", handleMouseUp);
    },
  };
}
