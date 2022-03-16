import type { Readable } from "svelte/store";
import { get, derived } from "svelte/store";

/**
 * Creates a new store from an existing store, which value is debounced:
 *
 * ```svelte
 * <script lang="ts">
 *   const counter = writable(0);
 *
 *   const debouncedCounter = debounced(counter, 300);
 * </script>
 *
 * <div>
 *   Debounced counter: {$debouncedCounter}
 * </div>
 *
 * <div on:click={() => $counter += 1}>
 *   Increment
 * </div>
 * ```
 *
 * In the above example if the user clicks 3 times and then waits for 300ms
 * the html will display "Debounced counter: 0" then "Debounced counter: 3"
 * without the other values.
 */
export function debounced<Value>(
  store: Readable<Value>,
  time: number
): Readable<Value> {
  let initialized = false;

  return derived(store, ($value, set) => {
    if (!initialized) {
      set($value);

      initialized = true;

      return;
    }

    const timeoutId = setTimeout(() => {
      set($value);
    }, time);

    return () => {
      clearTimeout(timeoutId);
    };
  });
}

/**
 * Creates a new store from an existing store, which the previous
 * value is saved and can be accessed at any time.
 *
 * ```svelte
 * <script lang="ts">
 *   const counter = writable(0);
 *
 *   const recordedCounter = recorded(counter);
 * </script>
 *
 * <div>
 *   Recorded counter: {$recordedCounter.curr} was {$recordedCounter.prev}
 * </div>
 *
 * <div on:click={() => $counter += 1}>
 *   Increment
 * </div>
 * ```
 *
 * In the above example, the first time the html will display "Recorded counter: 0 was undefined"
 * and "Recorded counter: 1 was 0" after the user clicks once, "Recorded counter: 2 was 1" the second time, etc...
 */
export function recorded<Value>(
  store: Readable<Value>
): Readable<{ curr: Value; prev: Value | undefined }> {
  let initialized = false;

  const recorded: Readable<{ curr: Value; prev: Value | undefined }> = derived(
    store,
    ($value, set) => {
      if (!initialized) {
        set({ curr: $value, prev: undefined });
        initialized = true;

        return;
      }

      const { curr } = get(recorded);

      set({ curr: $value, prev: curr });
    }
  );

  return recorded;
}

/**
 * Creates a new store from an existing store, which value is throttled:
 *
 * ```svelte
 * <script lang="ts">
 *   const counter = writable(0);
 *
 *   const throttledCounter = throttled(counter, 300);
 * </script>
 *
 * <div>
 *   Throttled counter: {$throttledCounter}
 * </div>
 *
 * <div on:click={() => $counter += 1}>
 *   Increment
 * </div>
 * ```
 *
 * In the above example if the user clicks 5 times, 3 times in 300ms and then 2 times,
 * the html will display "Throttled counter: 0" then "Throttled counter: 3" and finally
 * "Throttled counter: 5" after 300ms, without the other values.
 */
export function throttled<Value>(
  store: Readable<Value>,
  time: number
): Readable<Value> {
  let lastTime: number | undefined;

  return derived(store, (value, set) => {
    const now = Date.now();

    if (!lastTime || now - lastTime > time) {
      set(value);
      lastTime = now;
    } else {
      const timeoutId = setTimeout(() => {
        set(value);
      }, time);

      return () => clearTimeout(timeoutId);
    }
  });
}
