import type { Executor } from "./executors";
import type { SubscriptionId } from "./subscription";
import type { Destroyable } from "./types";

export abstract class Agent<Input, Output, Message> implements Destroyable {
  readonly executor: Executor<Output>;

  constructor(executor: Executor<Output>) {
    this.executor = executor;

    this.executor.registerDestroyableResource(this);
  }

  abstract onInput(input: Input, id: SubscriptionId): void;

  /** Used for internal updates only */
  update(_message: Message): void {
    // NOOP
  }

  /**
   * Called when a connection occurs, typically used to handle subscribers.
   *
   * The agent must return `true` if the connection is accepted, `false` otherwise
   */
  onConnection(_id: SubscriptionId): boolean {
    return false;
  }

  /** Called when a connection is over, typically used to handle subscribers */
  onDisconnection(_id: SubscriptionId): void {
    // NOOP
  }

  /** Called when the context exector is destroyed, typically used to cleanup subscribers */
  destroy(): void {
    // NOOP
  }
}

export type AgentConstructor<Input, Output, Message> = {
  new (executor: Executor<Output>): Agent<Input, Output, Message>;
};
