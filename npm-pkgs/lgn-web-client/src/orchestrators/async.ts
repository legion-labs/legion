import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export type AsyncInitValue<Data> = {
  data?: Data | null;
  loading?: boolean;
  error?: unknown;
};

export type AsyncOrchestrator<Data> = {
  loading: Writable<boolean>;
  error: Writable<unknown>;
  data: Writable<Data | null>;
  run(promise: () => Promise<Data>): Promise<Data>;
};

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
 * ## Example
 *
 * ```
 * const basicStoreOrchestrator = createAsyncStoreOrchestrator<string>();
 * const { data, error } = basicStoreOrchestrator;
 *
 * assert($data === null);
 *
 * await basicStoreOrchestrator.run(() => Promise.resolve("Hello"));
 *
 * assert($data === "Hello");
 *
 * try {
 *   await basicStoreOrchestrator.run(() => Promise.reject("Oh no..."));
 * } catch {}
 *
 * assert($data === null);
 * assert($error === "Oh no...");
 * ```
 */
export function createAsyncStoreOrchestrator<Data>(
  initValue: AsyncInitValue<Data> = {}
): AsyncOrchestrator<Data> {
  return {
    loading: writable(("loading" in initValue && initValue.loading) || false),

    error: writable("error" in initValue ? initValue.error : null),

    data: writable(("data" in initValue && initValue.data) || null),

    /**
     * Run the provided [async thunk](https://en.wikipedia.org/wiki/Thunk)
     * and populate the stores accordingly.
     */
    async run(promise) {
      this.loading.set(true);

      let newData: Data;

      try {
        newData = await promise();

        this.data.set(newData);

        this.error.set(null);
      } catch (error) {
        this.data.set(null);

        this.error.set(error);

        throw error;
      } finally {
        this.loading.set(false);
      }

      return newData;
    },
  };
}

export type AsyncListOrchestrator<Data> = AsyncOrchestrator<Data> & {
  loadMore(promise: () => Promise<Data>): Promise<Data>;
};

export function createAsyncStoreListOrchestrator<
  Data extends unknown[]
>(): AsyncListOrchestrator<Data> {
  return {
    ...createAsyncStoreOrchestrator<Data>(),

    /**
     * Unlike the `run` function, `loadMore` will not replace but rather append
     * the promise result to the data.
     *
     * `loadMore` returns _only the appended results_.
     *
     * On error the data will be preserved.
     */
    async loadMore(promise) {
      this.loading.set(true);

      let appendedData: Data;

      try {
        appendedData = await promise();

        this.data.update(
          (currentData): Data =>
            (currentData
              ? [...currentData, ...appendedData]
              : appendedData) as Data
        );

        this.error.set(null);
      } catch (error) {
        this.error.set(error);

        throw error;
      } finally {
        this.loading.set(false);
      }

      return appendedData;
    },
  };
}
