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

    if (
      typeof accept === "string"
        ? accept !== dndStoreValue.type
        : !accept.includes(dndStoreValue.type)
    ) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragover", {
        detail: { item: dndStoreValue.item, originalEvent: event },
      })
    );
  };

  const onDragEnter = (event: DragEvent) => {
    event.preventDefault();

    const dndStoreValue = get(store);

    if (!dndStoreValue) {
      return;
    }

    if (
      typeof accept === "string"
        ? accept !== dndStoreValue.type
        : !accept.includes(dndStoreValue.type)
    ) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragenter", {
        detail: { item: dndStoreValue.item, originalEvent: event },
      })
    );
  };

  const onDragLeave = (event: DragEvent) => {
    event.preventDefault();

    const dndStoreValue = get(store);

    if (!dndStoreValue) {
      return;
    }

    if (
      typeof accept === "string"
        ? accept !== dndStoreValue.type
        : !accept.includes(dndStoreValue.type)
    ) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-dragleave", {
        detail: { item: dndStoreValue.item, originalEvent: event },
      })
    );
  };

  const onDrop = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();

    store.set(null);

    const payload = event.dataTransfer?.getData("text/plain");

    if (!payload) {
      return;
    }

    const { item, type } = JSON.parse(payload);

    if (typeof accept === "string" ? accept !== type : !accept.includes(type)) {
      return;
    }

    element.dispatchEvent(
      new CustomEvent("dnd-drop", { detail: { item, originalEvent: event } })
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

    event.dataTransfer?.setData("text/plain", JSON.stringify({ type, item }));
    store.set({
      mousePosition: { x: event.clientX, y: event.clientY },
      item,
      type,
    });
  };

  const onDragEnd = (_event: DragEvent) => {
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
