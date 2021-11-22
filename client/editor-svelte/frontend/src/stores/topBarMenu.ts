import { derived, writable, Writable } from "svelte/store";

export const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Layer" },
  { id: 4, title: "Document" },
  { id: 5, title: "View" },
  { id: 6, title: "Help" },
] as const;

// Obviously not meant to be used as is in production
// as the menu might become dynamic at one point
export type Id = typeof menus[number]["id"];

export const openedMenuId: Writable<Id | null> = writable(null);

export const isOpen = derived(openedMenuId, ($openedMenuId) => !!$openedMenuId);

export const close = () => openedMenuId.set(null);

export const set = (id: Id) => openedMenuId.set(id);
