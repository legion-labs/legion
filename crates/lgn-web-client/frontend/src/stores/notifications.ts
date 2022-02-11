import { Writable } from "../lib/store";

export type Notification = {
  type: "success" | "warning" | "error";
  title: string;
  message: string;
  timeout?: number;
};

type Value = Notification & {
  close(): void;
  started: number;
  timeout: number;
  percentage: number;
};

const intervalMs = 16;

export default class extends Writable<Record<symbol, Value>> {
  #timeout: number;

  constructor(timeout = 5_000) {
    super({});

    this.#timeout = timeout;
  }

  push(key: symbol, value: Notification) {
    this.update((notifications) => {
      if (key in notifications) {
        return notifications;
      }

      const timeout =
        typeof value.timeout === "number" ? value.timeout : this.#timeout;

      const update = this.update.bind(this);

      const intervalId = setInterval(() => {
        this.update((notifications) => {
          const notification = notifications[key];

          const percentage =
            100 - (100 * (Date.now() - notification.started)) / timeout;

          return {
            ...notifications,
            [key]: { ...notification, percentage },
          };
        });
      }, intervalMs);

      const timeoutId = setTimeout(() => {
        this.update((notifications) => {
          clearInterval(intervalId);

          const { [key]: _, ...restNotifications } = notifications;

          return restNotifications;
        });
      }, timeout);

      return {
        ...notifications,
        [key]: {
          ...value,
          started: Date.now(),
          timeout,
          percentage: 100,
          close() {
            clearTimeout(timeoutId);
            clearInterval(intervalId);

            update((notifications) => {
              const { [key]: _, ...restNotifications } = notifications;

              return restNotifications;
            });
          },
        },
      };
    });
  }
}
