import { Agent } from "../agent";
import type { SubscriptionId } from "../subscription";

export class Mpsc<T> extends Agent<T, T, void> {
  #subscriptionId: SubscriptionId | null = null;

  override onConnection(id: SubscriptionId): boolean {
    // TODO: Log errors
    if (this.#subscriptionId) {
      return false;
    }

    this.#subscriptionId = id;

    return true;
  }

  override onDisconnection(_id: SubscriptionId): void {
    this.#subscriptionId = null;
  }

  onInput(input: T, id: SubscriptionId): void {
    // TODO: Log errors
    if (!this.#subscriptionId || this.#subscriptionId !== id) {
      return;
    }

    // Input is output for Mpsc
    this.executor.respondTo(this.#subscriptionId, input);
  }
}
