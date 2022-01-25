import log from "./log";

/**
 * Takes a function a return a function that acts similarly but is debounced
 * @param f The debounced function
 * @param ms How long the provided function should be debounced
 * @returns The provided function, debounced
 */
export function debounce<Args extends unknown[]>(
  f: (...args: Args) => void,
  ms: number
) {
  let timeout: ReturnType<typeof setTimeout> | null;

  return (...args: Args) => {
    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(() => {
      timeout = null;
      f(...args);
    }, ms);
  };
}

// TODO: This function can freeze the browser when misused, let's try to get rid of it
/**
 * Tries to call a function n times. If it succeeds the resulting promise is returned
 * @param f The function to call n times
 * @param n Amount of tries, if null or not provided tries forever
 * @returns The result of the succeeding function
 */
export async function retry<T>(
  f: () => Promise<T>,
  n: number | null = null
): Promise<T> {
  try {
    // We eagerly consume the promise and catch if it fails
    return await f();
  } catch (error) {
    if (n === null) {
      return retry(f, n);
    }

    if (n <= 0) {
      throw error;
    }

    n--;

    return retry(f, n);
  }
}

/**
 * Sleeps for n ms, this function uses `setTimeout` under the hood.
 *
 * Unlike traditional promises this one _can_ be aborted/cancelled.
 *
 * Doesn't throw.
 */
export function sleep(ms: number) {
  let abort = () => {
    log.warn("`abort` function not implemented by the promise builder");
  };

  if (ms <= 0) {
    return {
      abort() {
        // No promise to abort
      },
      promise: Promise.resolve(),
    };
  }

  const promise = new Promise((resolve) => {
    const timeoutId = setTimeout(() => {
      resolve(undefined);
    }, ms);

    abort = () => clearTimeout(timeoutId);
  });

  return { abort, promise };
}
