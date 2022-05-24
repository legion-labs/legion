/**
 * Debounce an event.
 */
export function debounce(
  cb: (event: Event) => void,
  time: number
): { (event: Event): void; clear(): void } {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  function clear() {
    if (timeoutId) {
      clearTimeout(timeoutId);

      timeoutId = null;
    }
  }

  function debounced(event: Event) {
    clear();

    timeoutId = setTimeout(() => {
      timeoutId = null;

      cb(event);
    }, time);
  }

  debounced.clear = clear;

  return debounced;
}
