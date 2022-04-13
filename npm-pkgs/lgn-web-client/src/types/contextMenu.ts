export type ItemEntry = {
  type: "item";
  action: string;
  label: string;
  tag?: "danger";
};

export type SeparatorEntry = { type: "separator" };

export type Entry = ItemEntry | SeparatorEntry;

export const contextMenuEventName = "contextmenu-action";

export type ContextMenuEventName = typeof contextMenuEventName;

export type ContextMenuEvent<
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
): ContextMenuEvent<Name, EntryRecord> {
  return new CustomEvent<ContextMenuActionDetail<EntryRecord>>(
    contextMenuEventName,
    {
      detail: { close, entrySetName, action },
    }
  );
}

export type EventHandler<
  Name extends string,
  EntryRecord extends Record<Name, unknown>
> = (event: ContextMenuEvent<Name, EntryRecord>) => Promise<void> | void;

/** Allows to "subscribe" to a specific entry set */
export function filterContextMenuEvents<
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

    return handler(event as ContextMenuEvent<Name, Pick<EntryRecord, Name>>);
  };
}
