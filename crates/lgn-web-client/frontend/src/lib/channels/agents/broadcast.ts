import { Agent } from "../agent";
import type { SubscriptionId } from "../subscription";

export class Broadcast<T> extends Agent<T, T, void> {
  #subscriptionIds: Set<SubscriptionId> = new Set();

  override onConnection(id: SubscriptionId): boolean {
    const previousSize = this.#subscriptionIds.size;

    this.#subscriptionIds.add(id);

    return this.#subscriptionIds.size > previousSize;
  }

  override onDisconnection(id: SubscriptionId): void {
    this.#subscriptionIds.delete(id);
  }

  onInput(input: T, _id: SubscriptionId): void {
    this.#subscriptionIds.forEach((subscriptionId) => {
      // Input is output for Broadcast
      this.executor.respondTo(subscriptionId, input);
    });
  }
}
