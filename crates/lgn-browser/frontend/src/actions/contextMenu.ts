import ContextMenuStore from "../stores/contextMenu";

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
    options:
      | string
      | {
          name: Name;
          /** @deprecated Use the component states or Svelte stores instead of a payload */
          payload: () => EntryRecord[Name];
        }
  ) {
    function listener(event: MouseEvent) {
      // In dev mode `Ctrl + Right Click` will open the default
      // context menu for dev purpose.
      if (import.meta.env.DEV && event.ctrlKey) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      const { name, payload } =
        typeof options === "string"
          ? { name: options, payload: () => undefined }
          : options;

      // TODO: We might get rid of the `payload` logic at one point
      window.dispatchEvent(
        new CustomEvent("custom-contextmenu", {
          detail: {
            name,
            payload,
            originalEvent: event,
          },
        })
      );
    }

    element.addEventListener("contextmenu", listener);

    return {
      destroy() {
        element.removeEventListener("contextmenu", listener);
      },
    };
  };
}
