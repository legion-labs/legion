import { Store as ContextMenuStore } from "../stores/contextMenu";

/**
 * Builds a type safe `contextMenu` action that automatically uses
 * the proper context menu (entries) on right click.
 *
 * The provided context menu name must have been registered using
 * the `contextMenuStore.register` function.
 */

export default function buildContextMenu<
  EntryRecord extends Record<string, unknown>
>(contextMenuStore: ContextMenuStore<EntryRecord>) {
  return function contextMenu<Name extends keyof EntryRecord>(
    element: HTMLElement,
    { name, payload }: { name: Name; payload: EntryRecord[Name] }
  ) {
    function listener() {
      contextMenuStore.setActiveEntrySet(name, payload);
    }

    element.addEventListener("contextmenu", listener);

    return {
      destroy() {
        element.removeEventListener("contextmenu", listener);
      },
    };
  };
}
