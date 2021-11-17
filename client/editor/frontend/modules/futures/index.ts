export function debounce<This, Args extends unknown[]>(
  f: (this: This, ...args: Args) => void,
  ms: number,
  immediate: boolean
) {
  let timeout: ReturnType<typeof setTimeout> | null;

  return function (this: This, ...args: Args) {
    // TODO: By using the composition API we should be able to get rid of the `this` references
    // in the `<script>` tags, that should make the following part useless.
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const context = this;

    const later = function () {
      timeout = null;

      if (!immediate) {
        f.apply(context, args);
      }
    };

    const callNow = immediate && !timeout;

    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(later, ms);

    if (callNow) {
      f.apply(context, args);
    }
  };
}

export function retryForever<T>(f: () => Promise<T>) {
  return retry(-1, f);
}

export async function retry<T>(
  maxRetries: number,
  f: () => Promise<T>
): Promise<T> {
  try {
    return await f();
  } catch (error) {
    if (maxRetries === 0) {
      throw error;
    }

    if (maxRetries > 0) {
      maxRetries--;
    }

    return retry(maxRetries, f);
  }
}
