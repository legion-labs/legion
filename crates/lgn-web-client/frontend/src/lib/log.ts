/**
 * Poor man, dead simple, log module.
 * Exposes functions to set the log level and to log to the browser console.
 *
 * The logger easily allows for log filtering by level and namespace(s).
 *
 * Doesn't sync with any API.
 */

export type Level = "error" | "warn" | "info" | "debug" | "trace";

const levels: Level[] = ["error", "warn", "info", "debug", "trace"];

function levelColor(level: Level) {
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

declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace globalThis {
    // eslint-disable-next-line no-var
    var __LOG__: { level: Level; namespace: RegExp } | null;
  }
}

const localStorageKey = "__LOG__";

/** Inits the log system, should be called in the `index` */
function init() {
  if (typeof localStorage !== "undefined") {
    const log = localStorage.getItem(localStorageKey);

    globalThis.__LOG__ = log ? JSON.parse(log) : null;
  } else {
    globalThis.__LOG__ = null;
  }
}

/** Set the log level and namespace to listen to */
function set(level: Level, namespace = /.*/) {
  const log = { level, namespace };

  if (typeof localStorage !== "undefined") {
    localStorage.setItem(localStorageKey, JSON.stringify(log));
  }

  globalThis.__LOG__ = log;
}

/** Stop displaying any log */
function hush() {
  globalThis.__LOG__ = null;

  if (typeof localStorage !== "undefined") {
    localStorage.removeItem(localStorageKey);
  }
}

/** Log function that accepts a level, a message, and optionally a namespace to log into */
function log(
  level: Level,
  ...args: [namespace: string, message: unknown] | [message: unknown]
) {
  if (!globalThis.__LOG__) {
    return;
  }

  const { level: requestedLevel, namespace: requestedNamespace } =
    globalThis.__LOG__;

  const namespace = args.length === 2 ? args[0] : "";

  const message = args.length === 2 ? args[1] : args[0];

  if (
    levelPriority(level) <= levelPriority(requestedLevel) &&
    requestedNamespace?.test(namespace)
  ) {
    // eslint-disable-next-line no-console
    console.log(
      `[%c${new Date().toISOString()} %c${level.toUpperCase()}%c${
        !namespace.length ? "" : ` ${namespace}`
      }]`,
      "color: purple",
      `color: ${levelColor(level)}`,
      "color: black",
      message
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

export default { init, log, set, hush, json, ...loggers };
