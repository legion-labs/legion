import { writable } from "svelte/store";

/**
 * Simple store for async data (used typically to execute gRPC requests).
 * It will expose several states that can be reused in multiple components
 * and a `run` function that's supposed to be run _once_.
 * The `run` function can be used in an [#await block](https://svelte.dev/tutorial/await-blocks)
 * to keep the code more idiomatic.
 * `run` will also throw if an error occurs so don't forget to `catch` it.
 *
 * @param promise The promise to run and resolve
 * @returns An object containing several states, including the resolved data, errors if any, and a loading state
 */
export default function asyncData<T>(promise: () => Promise<T>) {
  const loading = writable(false);
  const error = writable<unknown | null>(null);
  const data = writable<T | null>(null);

  async function run() {
    loading.set(true);
    error.set(null);

    let newData = await promise();

    try {
      newData = await promise();
      data.set(newData);
    } catch (e) {
      error.set(e);
      throw e;
    }

    loading.set(false);

    return newData;
  }

  return { data, loading, error, run };
}
