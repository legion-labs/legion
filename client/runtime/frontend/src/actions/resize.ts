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
