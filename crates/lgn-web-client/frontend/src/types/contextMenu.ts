export type ItemEntry = {
  type: "item";
  action: string;
  label: string;
  tag?: "danger";
};

export type SeparatorEntry = { type: "separator" };

export type Entry = ItemEntry | SeparatorEntry;

// The `as const` casting is not necessary in this case
// but it prevents the type to be inferred as `string`
// in some older versions of TS
export const eventName = "contextmenu-action" as const;

export type EventName = typeof eventName;

export type Event<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
> = CustomEvent<ContextMenuActionDetail<EntryRecord>>;

export function buildCustomEvent<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
>(
  close: () => void,
  entrySetName: Name,
  action: string
): Event<Name, EntryRecord> {
  return new CustomEvent<ContextMenuActionDetail<EntryRecord>>(eventName, {
    detail: { close, entrySetName, action },
  });
}

export type EventHandler<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
> = (event: Event<Name, EntryRecord>) => Promise<void> | void;

/** Auto close the context menu before action is triggered */
export function autoClose<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
>(handler: EventHandler<Name, EntryRecord>): EventHandler<Name, EntryRecord> {
  return function innerHandler(event) {
    event.detail.close();

    return handler(event);
  };
}

/** Allows to "subscribe" to a specific entry set */
export function select<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
>(
  handler: EventHandler<Name, Pick<EntryRecord, Name>>,
  ...entrySetNames: Name[]
): EventHandler<Name, EntryRecord> {
  return function innerHandler(event) {
    if (!entrySetNames.includes(event.detail.entrySetName as Name)) {
      return;
    }

    return handler(event as Event<Name, Pick<EntryRecord, Name>>);
  };
}
