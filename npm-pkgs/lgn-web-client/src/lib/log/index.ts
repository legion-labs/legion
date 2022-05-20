/**
 * Poor man, dead simple, log module.
 * Exposes functions to set the log level and to log to the browser console.
 *
 * The logger easily allows for log filtering by level and namespace(s).
 *
 * Doesn't sync with any API.
 */
import type { NonEmptyArray } from "../array";

export type Level = "error" | "warn" | "info" | "debug" | "trace";

const levels: Level[] = ["error", "warn", "info", "debug", "trace"];

/** The name of the dispatched event, usefule when implementing a transport */
export const eventName = "message";

export type Message = {
  date: Date;
  level: Level;
  namespace: string | null;
  message: unknown;
};

export interface Transport extends EventTarget {
  dispose(): void;
  /** Turns off the transport, use `set` to turn it on */
  hush(): void;
  /** Sets the transport log level and watched namespace on the fly */
  set({ level, namespace }: { level?: Level; namespace?: RegExp | null }): void;
}

declare global {
  // eslint-disable-next-line no-var
  var __LOG__: {
    transports: NonEmptyArray<Transport>;
  } | null;
}

/** Inits the log system, should be called in the `index` */
function init(transports: NonEmptyArray<Transport>) {
  globalThis.__LOG__ = { transports };
}

function dispose() {
  if (!globalThis.__LOG__) {
    return;
  }

  globalThis.__LOG__.transports.forEach((transport) => {
    transport.dispose();
  });
}

/** Log function that accepts a level, a message, and optionally a namespace to log into */
function log(
  level: Level,
  ...args: [namespace: string, message: unknown] | [message: unknown]
) {
  if (!globalThis.__LOG__) {
    return;
  }

  const { transports } = globalThis.__LOG__;

  const namespace = args.length === 2 ? args[0] : "";

  const message = args.length === 2 ? args[1] : args[0];

  for (const transport of transports) {
    transport.dispatchEvent(
      new CustomEvent(eventName, {
        detail: {
          date: new Date(),
          level,
          namespace: namespace || null,
          message,
        },
      })
    );
  }
}

/** Automatically serializes templates' expressions using `JSON.stringify` */
function json(stringParts: TemplateStringsArray, ...expressions: unknown[]) {
  return stringParts.reduce(
    (acc, part, index) =>
      `${acc}${part}${
        index === stringParts.length - 1
          ? ""
          : JSON.stringify(expressions[index])
      }`,
    ""
  );
}

const loggers = levels.reduce(
  (loggers, level) => ({
    ...loggers,
    [level]: (
      ...args: [namespace: string, message: unknown] | [message: unknown]
    ) => log(level, ...args),
  }),
  {} as {
    [Key in Level]: (
      ...args: [namespace: string, message: unknown] | [message: unknown]
    ) => void;
  }
);

export default { init, log, json, dispose, ...loggers };
