import type { Bus } from "./bus";
import { MainExecutorBus, WorkerExecutorBus } from "./bus";
import type { SubscriptionId } from "./subscription";
import type { Destroyable } from "./types";

/**
 * An executor is used to wrap an `Agent` and make it usable in any context.
 * The two existing contexts (for now) are "main", that is the main/ui thread
 * and "web worker".
 */
export interface Executor<Output> extends Destroyable {
  bus: Bus<SubscriptionId, Output>;

  respondTo(id: SubscriptionId, message: Output): void;

  registerDestroyableResource(resource: Destroyable): void;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class MainExecutor implements Executor<any> {
  #destroyableResources: Destroyable[] = [];

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  bus = new MainExecutorBus<SubscriptionId, any>();

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  respondTo(id: SubscriptionId, message: any): void {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.bus.send(id, message);
  }

  registerDestroyableResource(resource: Destroyable): void {
    this.#destroyableResources.push(resource);
  }

  destroy(): void {
    this.#destroyableResources.forEach((resource) => resource.destroy());
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class WorkerExecutor implements Executor<any> {
  #destroyableResources: Destroyable[] = [];

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  bus = new WorkerExecutorBus<SubscriptionId, any>();

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  respondTo(id: SubscriptionId, message: any): void {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.bus.send(id, message);
  }

  registerDestroyableResource(resource: Destroyable): void {
    this.#destroyableResources.push(resource);
  }

  destroy(): void {
    this.#destroyableResources.forEach((resource) => resource.destroy());
  }
}
