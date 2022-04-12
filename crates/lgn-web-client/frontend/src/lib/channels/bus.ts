import type { Opaque } from "type-fest";
import { v4 as uuid } from "uuid";

import type { Subscriber } from "./types";

export type BusSubscriptionId = Opaque<string, "BusSubscriptionId">;

function createBusSubscriptionId(): BusSubscriptionId {
  return uuid() as BusSubscriptionId;
}

/**
 * "Hot" buses used internally by executors to handle messages.
 * Typically, the main/ui executor would use an `EventTarget` while
 * the web-worker one will use web workers.
 */
export interface Bus<Id extends string, T> {
  /** Send a message to the bus using only the provided `Id` */
  send(id: Id, message: T): void;

  /**
   * Subscribe to the bus' messages, an id is returned that can be used with the `unsubscribe` method.
   *
   * Bus' subscription is said "hot", that is, a new subscriber will _not_ receive previous message(s)
   * it will not be called before the next event is dispatched.
   */
  subscribe(id: Id, subscriber: Subscriber<T>): BusSubscriptionId;

  /** Unsubscribe one subscriber to the bus' messages */
  unsubscribe(busSubscriptionId: BusSubscriptionId): void;

  /** Destroys the bus */
  destroy(): void;
}

export class MainExecutorBus<Id extends string, T>
  extends EventTarget
  implements Bus<Id, T>
{
  #eventName: string;
  #listeners: Map<
    BusSubscriptionId,
    { id: Id; callback: (event: CustomEvent<T>) => void }
  > = new Map();

  constructor(eventName = "main-executor-bus-message-event") {
    super();

    this.#eventName = eventName;
  }

  send(id: Id, message: T): void {
    this.dispatchEvent(
      new CustomEvent(this.#getEventName(id), { detail: message })
    );
  }

  subscribe(id: Id, subscriber: Subscriber<T>): BusSubscriptionId {
    const newBusSubscriptionId = createBusSubscriptionId();

    const listenerCallback = ({ detail: message }: CustomEvent<T>) => {
      subscriber(message);
    };

    this.#listeners.set(newBusSubscriptionId, {
      id,
      callback: listenerCallback,
    });

    this.addEventListener(
      this.#getEventName(id),
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
      listenerCallback as any
    );

    return newBusSubscriptionId;
  }

  unsubscribe(busSubscriptionId: BusSubscriptionId): void {
    const listener = this.#listeners.get(busSubscriptionId);

    if (!listener) {
      return;
    }

    this.removeEventListener(
      this.#getEventName(listener.id),
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
      listener.callback as any
    );

    this.#listeners.delete(busSubscriptionId);
  }

  destroy(): void {
    for (const [, { id, callback }] of this.#listeners) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
      this.removeEventListener(this.#getEventName(id), callback as any);
    }

    this.#listeners = new Map();
  }

  #getEventName(id: Id) {
    return `${this.#eventName}-${id}`;
  }
}

// TODO: Implement the WorkerExecutorBus
export class WorkerExecutorBus<Id extends string, T> implements Bus<Id, T> {
  send(_id: Id, _message: T): void {
    // NOOP
  }

  subscribe(_id: Id, _subscriber: Subscriber<T>): BusSubscriptionId {
    // NOOP

    return createBusSubscriptionId();
  }

  unsubscribe(_busSubscriptionId: BusSubscriptionId): void {
    // NOOP
  }

  destroy(): void {
    // NOOP
  }
}
