export default function resize(
  node: Node,
  listener: (rect: DOMRectReadOnly) => void
) {
  const observer = new ResizeObserver(([entry]) => {
    listener(entry.contentRect);
  });

  if (node instanceof Element) {
    observer.observe(node as Element);
  }

  return {
    destroy() {
      observer.disconnect();
    },
  };
}
