import { eventName } from ".";
import type { Level, Message, Transport } from ".";
import type { NotificationsStore } from "../../stores/notifications";

function levelPriority(level: Level) {
  switch (level) {
    case "error":
      return 0;
    case "warn":
      return 1;
    case "info":
      return 2;
    case "debug":
      return 3;
    case "trace":
      return 4;
  }
}

export abstract class TransportBase extends EventTarget implements Transport {
  protected listener: (event: Event) => void;

  constructor(protected config: { level: Level; namespace?: RegExp | null }) {
    super();

    this.listener = (event: Event) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }

      const { detail: message } = event as CustomEvent<Message>;

      if (!this.isMessageAccepted(message)) {
        return;
      }

      this.handleMessage(message);
    };

    this.addEventListener(eventName, this.listener);
  }

  abstract handleMessage(message: Message): void;

  protected isMessageAccepted(message: Message): boolean {
    return (
      levelPriority(message.level) <= levelPriority(this.config.level) &&
      (!this.config.namespace
        ? true
        : this.config.namespace.test(message.namespace || ""))
    );
  }

  set({
    level,
    namespace,
  }: {
    level?: Level;
    namespace?: RegExp | null;
  }): void {
    if (typeof level !== "undefined") {
      this.config.level = level;
    }

    if (typeof namespace !== "undefined") {
      this.config.namespace = namespace;
    }
  }

  hush(): void {
    // Noop
  }

  dispose(): void {
    this.removeEventListener(eventName, this.listener);
  }
}

function levelToColor(level: Level) {
  switch (level) {
    case "error":
      return "red";
    case "warn":
      return "orange";
    case "info":
      return "skyblue";
    case "debug":
      return "gray";
    case "trace":
      return "black";
  }
}

/** The "void" transport, does nothing on message */
export class VoidTransport extends EventTarget implements Transport {
  dispose(): void {
    // Noop
  }

  set(_: { namespace?: RegExp | null; level?: Level }): void {
    // Noop
  }

  hush(): void {
    // Noop
  }
}

/** The "console" transport, uses the console API to log messages */
export class ConsoleTransport extends TransportBase {
  override handleMessage({ date, level, message, namespace }: Message): void {
    // eslint-disable-next-line no-console
    console.log(
      `%c[%c${date.toISOString()} %c${level.toUpperCase()}%c${
        namespace ? ` ${namespace}` : ""
      }]`,
      "color: black",
      "color: purple",
      `color: ${levelToColor(level)}`,
      "color: black",
      message
    );
  }
}

function levelToNotificationType(level: Level) {
  switch (level) {
    case "debug":
    case "info":

    // eslint-disable-next-line no-fallthrough
    case "trace": {
      return "success";
    }

    case "error": {
      return "error";
    }

    case "warn": {
      return "warning";
    }
  }
}

/** The "notifications" transport, uses the console API to log messages */
export class NotificationsTransport extends TransportBase {
  #notificationsStore: NotificationsStore;

  constructor({
    notificationsStore,
    ...config
  }: {
    notificationsStore: NotificationsStore;
    level: Level;
    namespace?: RegExp;
  }) {
    super(config);

    this.#notificationsStore = notificationsStore;
  }

  override handleMessage({ date, level, message, namespace }: Message): void {
    this.#notificationsStore.push(Symbol(), {
      message: typeof message === "string" ? message : JSON.stringify(message),
      title: namespace
        ? `${namespace} - ${date.toISOString()}`
        : date.toISOString(),
      type: levelToNotificationType(level),
    });
  }
}
