/**
 * Builds a type safe `contextMenu` action that automatically uses
 * the proper context menu (entries) on right click.
 *
 * The provided context menu name must have been registered using
 * the `contextMenuStore.register` function.
 */

/** Alias of `unknown` used for explicitness */
export type JSONSerializable = unknown;

export default function buildContextMenu<
  Name extends string = string,
  Payload extends JSONSerializable = unknown
>() {
  return function contextMenu(
    element: HTMLElement,
    { name, payload }: { name: Name; payload?: Payload }
  ) {
    element.dataset.contextMenu = name;

    if (payload) {
      element.dataset.contextMenuPayload = JSON.stringify(payload);
    }
  };
}
