// Bare minimal drag and drop actions
// In the future we could use a bigger library
// or implement our own svelte adapter for
// https://github.com/react-dnd/react-dnd/tree/main/packages/dnd-core
import { derived, get } from "svelte/store";

import type { ActionReturn } from "../lib/action";
import type { DndStore } from "../stores/dnd";
import { createDndStore } from "../stores/dnd";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const defaultStore = createDndStore<any>();

export const isDragging = derived(
  defaultStore,
  ($defaultStore) => !!$defaultStore
);

export function dropzone<Item>(
  element: HTMLElement,
  {
    accept,
    store = defaultStore,
  }: { accept: string | string[]; store?: DndStore<Item> }
): ActionReturn {
  const onDragOver = (event: DragEvent) => {
    event.preventDefault();

    const dndStoreValue = get(store);

    if (!dndStoreValue) {
      return;
    }

    const value = dndStoreValue.find(({ type }) =>
      typeof accept === "string" ? accept !== type : accept.includes(type)
    );

    if (!value) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragover", {
        detail: { ...value, originalEvent: event },
      })
    );
  };

  const onDragEnter = (event: DragEvent) => {
    event.preventDefault();

    const dndStoreValue = get(store);

    if (!dndStoreValue) {
      return;
    }

    const value = dndStoreValue.find(({ type }) =>
      typeof accept === "string" ? accept !== type : accept.includes(type)
    );

    if (!value) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragenter", {
        detail: { ...value, originalEvent: event },
      })
    );
  };

  const onDragLeave = (event: DragEvent) => {
    event.preventDefault();

    const dndStoreValue = get(store);

    if (!dndStoreValue) {
      return;
    }

    const value = dndStoreValue.find(({ type }) =>
      typeof accept === "string" ? accept !== type : accept.includes(type)
    );

    if (!value) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragleave", {
        detail: { ...value, originalEvent: event },
      })
    );
  };

  const onDrop = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();

    const dndStoreValue = get(store);

    store.set(null);

    if (!dndStoreValue) {
      return;
    }

    const value = dndStoreValue.find(({ type }) =>
      typeof accept === "string" ? accept !== type : accept.includes(type)
    );

    if (!value) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-drop", {
        detail: { ...value, originalEvent: event },
      })
    );
  };

  element.addEventListener("dragenter", onDragEnter);
  element.addEventListener("dragleave", onDragLeave);
  element.addEventListener("dragover", onDragOver);
  element.addEventListener("drop", onDrop);

  return {
    destroy() {
      element.removeEventListener("dragenter", onDragEnter);
      element.removeEventListener("dragleave", onDragLeave);
      element.removeEventListener("dragover", onDragOver);
      element.removeEventListener("dragend", onDrop);
    },
  };
}

export function draggable<Item>(
  element: HTMLElement,
  {
    type,
    item,
    dropEffect,
    preview,
    store = defaultStore,
  }: {
    type: string;
    item: Item;
    dropEffect?: "none" | "copy" | "link" | "move";
    preview?: "none" | "custom" | "default";
    store?: DndStore<Item>;
  }
): ActionReturn {
  let img: HTMLImageElement | null = null;

  if (preview === "custom" || preview === "none") {
    img = document.createElement("img");
  }

  element.draggable = true;

  const onDragStart = (event: DragEvent) => {
    if (dropEffect && event.dataTransfer) {
      event.dataTransfer.dropEffect = dropEffect;
    }

    if (img && (preview === "custom" || preview === "none")) {
      event.dataTransfer?.setDragImage(img, 0, 0);
    }

    // TODO: This is not used anymore, should we drop it entirely or still keep some payload just in case?
    event.dataTransfer?.setData("text/plain", JSON.stringify({ type, item }));

    const value = {
      mousePosition: { x: event.clientX, y: event.clientY },
      item,
      type,
    };

    store.update((values) => (values ? [...values, value] : [value]));
  };

  const onDragEnd = (_event: DragEvent) => {
    // TODO: Remove only one value at a time?
    store.set(null);
  };

  element.addEventListener("dragstart", onDragStart);
  element.addEventListener("dragend", onDragEnd);

  return {
    destroy() {
      element.removeEventListener("dragstart", onDragStart);
      element.removeEventListener("dragend", onDragEnd);
    },
  };
}
