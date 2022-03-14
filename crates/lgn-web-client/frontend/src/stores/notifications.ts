import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export type Notification = {
  type: "success" | "warning" | "error";
  title: string;
  message: string;
  timeout?: number;
};

export type NotificationsValue = Notification & {
  close(): void;
  started: number;
  timeout: number;
  percentage: number;
};

export type NotificationsStore = Writable<
  Record<symbol, NotificationsValue>
> & {
  push(key: symbol, value: Notification): void;
};

const intervalMs = 16;

export function createNotificationsStore(
  requestedTimeout = 5_000
): NotificationsStore {
  return {
    ...writable<Record<symbol, NotificationsValue>>({}),

    push(key: symbol, value: Notification) {
      const update = this.update;

      this.update((notifications) => {
        if (key in notifications) {
          return notifications;
        }

        const timeout =
          typeof value.timeout === "number" ? value.timeout : requestedTimeout;

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
    },
  };
}
