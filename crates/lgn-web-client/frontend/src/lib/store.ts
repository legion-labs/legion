import { noop, safe_not_equal as safeNotEqual } from "svelte/internal";
import {
  StartStopNotifier,
  Subscriber,
  Unsubscriber,
  Updater,
} from "svelte/store";

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
