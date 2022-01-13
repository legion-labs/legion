import { Writable, writable } from "svelte/store";

export type InitAsyncStoreValue<Data> = {
  data?: Data | null;
  loading?: boolean;
  error?: unknown;
};

type SingleValueAsyncStore<Data> = {
  data: Writable<Data | null>;
  loading: Writable<boolean>;
  error: Writable<unknown>;
  /**
   * Run the provided [async thunk](https://en.wikipedia.org/wiki/Thunk)
   * and populate the stores accordingly.
   */
  run(promise: () => Promise<Data>): Promise<Data>;
};

type ListAsyncStore<Data extends unknown[]> = SingleValueAsyncStore<Data> & {
  /**
   * Unlike the `run` function, `loadMore` will not replace but rather append
   * the promise result to the data.
   *
   * `loadMore` returns _only the appended results_.
   *
   * On error the data will be preserved.
   */
  loadMore(promise: () => Promise<Data>): Promise<Data>;
};

export type AsyncStore<Data> = Data extends unknown[]
  ? ListAsyncStore<Data>
  : SingleValueAsyncStore<Data>;

// TODO: Add initial value support
/**
 * Simple store for async data (used typically to execute gRPC requests).
 * It will expose several states that can be reused in multiple components
 * and a `run` function which accepts a promise and that can be executed
 * as many times as needed.
 *
 * The value returned by the `run` function can be used in an
 * [#await block](https://svelte.dev/tutorial/await-blocks)
 * to keep the code more idiomatic.
 *
 * `run` will also throw if an error occurs so don't forget to `catch` it.
 *
 * Example:
 *
 * ```
 * const { data, error } = asyncStore<string>();
 *
 * console.assert($data === null);
 *
 * await basicAsyncStore.run(() => Promise.resolve("Hello"));
 *
 * console.assert($data === "Hello");
 *
 * try {
 *   await basicAsyncStore.run(() => Promise.reject("Oh no..."));
 * } catch {}
 *
 * console.assert($data === null);
 * console.assert($error === "Oh no...");
 * ```
 *
 * @param initValue - Defaults to "init", data, error, and even the loading state can be initialized using this param
 * @returns An object containing several states, including the resolved data, errors if any, and a loading state
 */
export default function asyncStore<Data extends unknown[]>(
  initValue?: InitAsyncStoreValue<Data>
): ListAsyncStore<Data>;
export default function asyncStore<Data>(
  initValue?: InitAsyncStoreValue<Data>
): SingleValueAsyncStore<Data>;
export default function asyncStore<Data extends unknown[]>(
  initValue: InitAsyncStoreValue<Data> = {}
): unknown {
  const loading = writable("loading" in initValue ? initValue.loading : false);
  const error = writable("error" in initValue ? initValue.error : null);
  const data = writable("data" in initValue ? initValue.data : null);

  async function run(promise: () => Promise<Data>) {
    loading.set(true);

    let newData: Data;

    try {
      newData = await promise();

      data.set(newData);

      error.set(null);
    } catch (e) {
      data.set(null);

      error.set(e);

      throw e;
    } finally {
      loading.set(false);
    }

    return newData;
  }

  async function loadMore(promise: () => Promise<Data>) {
    loading.set(true);

    let appendedData: Data;

    try {
      appendedData = await promise();

      data.update(
        (currentData): Data =>
          (currentData
            ? [...currentData, ...appendedData]
            : appendedData) as Data
      );

      error.set(null);
    } catch (e) {
      error.set(e);

      throw e;
    } finally {
      loading.set(false);
    }

    return appendedData;
  }

  return {
    data,
    loading,
    error,
    run,
    loadMore,
  };
}
