// Taken from https://svelte.dev/repl/0ace7a508bd843b798ae599940a91783?version=3.16.7
export default function clickOutside(node: Node, listener: () => void) {
  const handleClick = (event: MouseEvent) => {
    if (
      node &&
      event.target instanceof Node &&
      !node.contains(event.target) &&
      !event.defaultPrevented
    ) {
      listener();
    }
  };

  window.addEventListener("click", handleClick, true);

  return {
    destroy() {
      window.removeEventListener("click", handleClick, true);
    },
  };
}
