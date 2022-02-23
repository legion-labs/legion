declare type ContextMenuActionDetail<
  EntryRecord extends Record<string, unknown>
> = {
  [Name in keyof EntryRecord]: {
    /** Closes the context menu */
    close(): void;
    /** Name of the context menu entry set */
    entrySetName: Name;
    /** The action of the entry in the context menu (e.g.: `"rename"`, `"new"`, etc...) */
    action: string;
  };
}[keyof EntryRecord];

declare namespace svelte.JSX {
  interface DOMAttributes<T> {
    "onclick-outside"?: (
      event: CustomEvent<{ originalEvent: MouseEvent }> & {
        target: EventTarget & T;
      }
    ) => void;

    "onnavigation-change"?: (
      event: CustomEvent<number> & {
        target: EventTarget & T;
      }
    ) => void;

    "onnavigation-select"?: (
      event: CustomEvent<number | null> & {
        target: EventTarget & T;
      }
    ) => void;

    "onnavigation-rename"?: (
      event: CustomEvent<number | null> & {
        target: EventTarget & T;
      }
    ) => void;

    "onnavigation-remove"?: (
      event: CustomEvent<number | null> & {
        target: EventTarget & T;
      }
    ) => void;

    "oncustom-contextmenu"?: (
      event: CustomEvent<{
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        name: any;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        payload(): any;
        originalEvent: MouseEvent;
      }> & {
        target: EventTarget & T;
      }
    ) => void;

    "oncontextmenu-action"?: (
      // More permissive version of the event detail type
      event: CustomEvent<{
        close(): void;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        entrySetName: any;
        action: string;
      }> & {
        target: EventTarget & Window;
      }
    ) => void;

    "onrefresh-property"?: (
      event: CustomEvent<{ path: string; value: unknown }>
    ) => void;
  }
}
