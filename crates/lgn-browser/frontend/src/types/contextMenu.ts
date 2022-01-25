/* eslint-disable @typescript-eslint/no-namespace */
import { Phantom } from "./phantom";

export type ItemEntry<Payload> = Phantom<
  { type: "item"; action: string; label: string; tag?: "danger" },
  Payload
>;

export type SeparatorEntry<Payload> = Phantom<{ type: "separator" }, Payload>;

export type Entry<Payload> = ItemEntry<Payload> | SeparatorEntry<Payload>;

// The `as const` casting is not necessary in this case
// but it prevents the type to be inferred as `string`
// in some older versions of TS
export const eventName = "contextmenu-action" as const;

export type EventName = typeof eventName;

export type Detail<EntryRecord extends Record<string, unknown>> = {
  [Name in keyof EntryRecord]: {
    /** Closes the context menu */
    close(): void;
    /** Name of the context menu entry set */
    entrySetName: Name;
    /** The action of the entry in the context menu (e.g.: `"rename"`, `"new"`, etc...) */
    action: string;
    /** The payload attached to the action */
    payload: EntryRecord[Name];
  };
}[keyof EntryRecord];

export type Event<EntryRecord extends Record<string, unknown>> = CustomEvent<
  Detail<EntryRecord>
>;

export function buildCustomEvent<
  EntryRecord extends Record<string, unknown>,
  Name extends keyof EntryRecord
>(
  close: () => void,
  entrySetName: Name,
  action: string,
  payload: EntryRecord[Name]
): Event<EntryRecord> {
  return new CustomEvent<Detail<EntryRecord>>(eventName, {
    detail: { close, entrySetName, action, payload },
  });
}

export type EventHandler<EntryRecord extends Record<string, unknown>> = (
  event: Event<EntryRecord>
) => Promise<void> | void;

/** Auto close the context menu before action is trigered */
export function autoClose<EntryRecord extends Record<string, unknown>>(
  handler: EventHandler<EntryRecord>
): EventHandler<EntryRecord> {
  return function innerHandler(event) {
    event.detail.close();

    return handler(event);
  };
}

/** Allows to "subscribe" to a specific entry set */
export function select<
  EntryRecord extends Record<string, unknown>,
  Name extends keyof EntryRecord
>(
  handler: EventHandler<Pick<EntryRecord, Name>>,
  entrySetName: Name
): EventHandler<EntryRecord> {
  return function innerHandler(event) {
    if (entrySetName !== event.detail.entrySetName) {
      return;
    }

    return handler(event as Event<Pick<EntryRecord, Name>>);
  };
}
