import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

import type {
  FluentBase,
  ResolveFluentRecordVariablesOnly,
} from "../types/fluent";

export type NotificationRawPayload = {
  type: "raw";
  title: string;
  message: string;
};

export type NotificationL10nPayload<
  Fluent extends FluentBase,
  TitleId extends keyof Fluent,
  MessageId extends keyof Fluent
> = {
  type: "l10n";
  title: ResolveFluentRecordVariablesOnly<Fluent, TitleId>;
  message: ResolveFluentRecordVariablesOnly<Fluent, MessageId>;
};

export type Notification<
  Fluent extends FluentBase,
  TitleId extends keyof Fluent,
  MessageId extends keyof Fluent
> = {
  type: "success" | "warning" | "error";
  timeout?: number;
  payload:
    | NotificationRawPayload
    | NotificationL10nPayload<Fluent, TitleId, MessageId>;
};

export type NotificationsValue<
  Fluent extends FluentBase,
  TitleId extends keyof Fluent,
  MessageId extends keyof Fluent
> = Notification<Fluent, TitleId, MessageId> & {
  started: number;
  timeout: number;
  percentage: number | null;
  paused: boolean;
  intervalId: ReturnType<typeof setInterval>;
};

export type NotificationsStore<Fluent extends FluentBase> = Writable<
  Record<symbol, NotificationsValue<Fluent, keyof Fluent, keyof Fluent>>
> & {
  close(key: symbol): void;
  pause(key: symbol): void;
  resume(key: symbol): void;
  push<TitleId extends keyof Fluent, MessageId extends keyof Fluent>(
    key: symbol,
    value: Notification<Fluent, TitleId, MessageId>
  ): void;
};

const intervalMs = 16;

export function createNotificationsStore<Fluent extends FluentBase>(
  requestedTimeout = 5_000
): NotificationsStore<Fluent> {
  const notificationsStore = writable<
    Record<symbol, NotificationsValue<Fluent, keyof Fluent, keyof Fluent>>
  >({});

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

    push<TitleId extends keyof Fluent, MessageId extends keyof Fluent>(
      key: symbol,
      value: Notification<Fluent, TitleId, MessageId>
    ) {
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
