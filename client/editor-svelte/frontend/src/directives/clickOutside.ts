// Taken from https://svelte.dev/repl/0ace7a508bd843b798ae599940a91783?version=3.16.7
export default function clickOutside(node: Node) {
  const handleClick = (event: MouseEvent) => {
    if (
      node &&
      event.target instanceof Node &&
      !node.contains(event.target) &&
      !event.defaultPrevented
    ) {
      node.dispatchEvent(new CustomEvent("click-outside"));
    }
  };

  document.addEventListener("click", handleClick, true);

  return {
    destroy() {
      document.removeEventListener("click", handleClick, true);
    },
  };
}
