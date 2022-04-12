import type { Opaque } from "type-fest";
import { v4 as uuid } from "uuid";

import type { Agent } from "./agent";
import type { BusSubscriptionId } from "./bus";
import type { Executor } from "./executors";
import type { Destroyable, Subscriber } from "./types";

export type SubscriptionId = Opaque<string, "SubscriptionId">;

function createSubscriptionId(): SubscriptionId {
  return uuid() as SubscriptionId;
}

/** Handles a single subscription to an agent */
export class Subscription<Input, Output, Message> implements Destroyable {
  #agent: Agent<Input, Output, Message>;
  #id = createSubscriptionId();
  #busSubscriptionId: BusSubscriptionId | null = null;
  #executor: Executor<Output>;

  constructor(
    agent: Agent<Input, Output, Message>,
    subscriber: Subscriber<Output>
  ) {
    this.#agent = agent;
    this.#executor = agent.executor;

    this.#executor.registerDestroyableResource(this);

    const connectionAccepted = this.#agent.onConnection(this.#id);

    if (!connectionAccepted) {
      return;
    }

    this.#busSubscriptionId = this.#executor.bus.subscribe(
      this.#id,
      (message) => subscriber(message)
    );
  }

  send(message: Input): void {
    this.#agent.onInput(message, this.#id);
  }

  destroy() {
    if (this.#busSubscriptionId) {
      this.#executor.bus.unsubscribe(this.#busSubscriptionId);
    }
  }
}
