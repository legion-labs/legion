import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export type Notification = {
  type: "success" | "warning" | "error";
  title: string;
  message: string;
  timeout?: number;
};

export type NotificationsValue = Notification & {
  started: number;
  timeout: number;
  percentage: number | null;
  paused: boolean;
  intervalId: ReturnType<typeof setInterval>;
};

export type NotificationsStore = Writable<
  Record<symbol, NotificationsValue>
> & {
  close(key: symbol): void;
  pause(key: symbol): void;
  resume(key: symbol): void;
  push(key: symbol, value: Notification): void;
};

const intervalMs = 16;

export function createNotificationsStore(
  requestedTimeout = 5_000
): NotificationsStore {
  const notificationsStore = writable<Record<symbol, NotificationsValue>>({});

  return {
    ...notificationsStore,

    pause(key: symbol) {
      notificationsStore.update((notifications) => {
        const notification = notifications[key];

        if (!notification) {
          return notifications;
        }

        return {
          ...notifications,
          [key]: { ...notification, paused: true, percentage: null },
        };
      });
    },

    resume(key: symbol) {
      notificationsStore.update((notifications) => {
        const notification = notifications[key];

        if (!notification) {
          return notifications;
        }

        return {
          ...notifications,
          [key]: { ...notification, paused: false, started: Date.now() },
        };
      });
    },

    close(key: symbol) {
      notificationsStore.update((notifications) => {
        const { [key]: notification, ...remainingNotifications } =
          notifications;

        if (!notification) {
          return notifications;
        }

        clearInterval(notification.intervalId);

        return remainingNotifications;
      });
    },

    push(key: symbol, value: Notification) {
      notificationsStore.update((notifications) => {
        if (key in notifications) {
          return notifications;
        }

        const timeout =
          typeof value.timeout === "number" ? value.timeout : requestedTimeout;

        const intervalId = setInterval(() => {
          notificationsStore.update((notifications) => {
            const { [key]: notification, ...remainingNotifications } =
              notifications;

            if (!notification || notification.paused) {
              return notifications;
            }

            const percentage =
              100 -
              (100 * (Date.now() - notification.started)) /
                notification.timeout;

            if (Math.floor(percentage) === 0) {
              clearInterval(notification.intervalId);

              return remainingNotifications;
            }

            return {
              ...notifications,
              [key]: { ...notification, percentage },
            };
          });
        }, intervalMs);

        return {
          ...notifications,
          [key]: {
            ...value,
            started: Date.now(),
            timeout,
            percentage: 100,
            paused: false,
            intervalId,
          },
        };
      });
    },
  };
}
