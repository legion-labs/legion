import { BehaviorSubject, Observable } from "rxjs";
import { noop, safe_not_equal as safeNotEqual } from "svelte/internal";
import type {
  StartStopNotifier,
  Subscriber,
  Unsubscriber,
  Updater,
  Readable as SvelteReadable,
  Writable as SvelteWritable,
} from "svelte/store";
import { get, derived, writable } from "svelte/store";

/** A store orchestrator is an object that contains and orchestrate/manipulate store(s) but is not a store itself */
export interface Orchestrator {
  name: string;
}

type Invalidator<T> = (value?: T) => void;

type SubscribeInvalidate<T> = {
  subscribe: Subscriber<T>;
  invalidate: Invalidator<T>;
};

export class Readable<T> {
  protected subscriberQueue: {
    subscribeInvalidate: SubscribeInvalidate<T>;
    value: T;
  }[] = [];
  protected start: StartStopNotifier<T> = noop;
  protected stop: Unsubscriber | null = null;
  protected subscribers: Set<SubscribeInvalidate<T>> = new Set();

  value: T;

  constructor(value: T, start: StartStopNotifier<T> = noop) {
    this.value = value;
    this.start = start;
  }

  subscribe(run: Subscriber<T>, invalidate: Invalidator<T> = noop) {
    const subscriber: SubscribeInvalidate<T> = { subscribe: run, invalidate };

    this.subscribers.add(subscriber);

    if (this.subscribers.size === 1) {
      this.stop = this.start.call(this, this.set.bind(this)) || noop;
    }

    run(this.value);

    return () => {
      this.subscribers.delete(subscriber);

      if (this.subscribers.size === 0 && this.stop) {
        this.stop();
        this.stop = null;
      }
    };
  }

  protected set(newValue: T): void {
    if (!safeNotEqual(this.value, newValue)) {
      return;
    }

    this.value = newValue;

    if (this.stop) {
      const runQueue = !this.subscriberQueue.length;

      for (const subscriber of this.subscribers) {
        subscriber.invalidate();

        this.subscriberQueue.push({
          subscribeInvalidate: subscriber,
          value: this.value,
        });
      }

      if (runQueue) {
        for (const {
          subscribeInvalidate: { subscribe },
          value,
        } of this.subscriberQueue) {
          subscribe(value);
        }

        this.subscriberQueue.length = 0;
      }
    }
  }

  protected update(fn: Updater<T>): void {
    this.set(fn(this.value));
  }
}

export class Writable<T> extends Readable<T> {
  constructor(value: T, start: StartStopNotifier<T> = noop) {
    super(value, start);
  }

  override set(newValue: T): void {
    super.set(newValue);
  }

  override update(fn: Updater<T>): void {
    super.update(fn);
  }
}

/**
 * Takes a readable/writable store and turns it into a `BehaviorSubject`
 *
 * When used with the auto subscribed operator `$` the value _can_ be
 * `undefined` at runtime if you pipe the returned subject to some
 * async operators (like `delay` for instance).
 */
export function fromStore<Value>(store: SvelteReadable<Value>) {
  const subject = new BehaviorSubject<Value>(null as unknown as Value);

  store.subscribe((value) => subject.next(value));

  return subject;
}

/** Takes an Observable and turns into a Svelte's `Writable` store */
export function toStore<Value>(observable: Observable<Value>) {
  const store = writable<Value>();

  observable.subscribe((value) => store.set(value));

  return store;
}

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
  store: SvelteReadable<Value>,
  time: number
): SvelteReadable<Value> {
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
  store: SvelteReadable<Value>
): SvelteReadable<{ curr: Value; prev: Value | undefined }> {
  let initialized = false;

  const recorded: SvelteReadable<{ curr: Value; prev: Value | undefined }> =
    derived(store, ($value, set) => {
      if (!initialized) {
        set({ curr: $value, prev: undefined });
        initialized = true;

        return;
      }

      const { curr } = get(recorded);

      set({ curr: $value, prev: curr });
    });

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
  store: SvelteReadable<Value>,
  time: number
): SvelteReadable<Value> {
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
