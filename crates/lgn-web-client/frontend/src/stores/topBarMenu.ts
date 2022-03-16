import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

// Not meant to be used as is in production
// as the menu might become dynamic at one point
export type Id = typeof menus[number]["id"];

export type TopBarMenuValue = Id | null;

export type TopBarMenuStore = Writable<TopBarMenuValue> & {
  close(): void;
  set(id: Id): void;
};

export const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Layer" },
  { id: 4, title: "Document" },
  { id: 5, title: "View" },
  { id: 6, title: "Help" },
] as const;

function createTopBarMenuStore(): TopBarMenuStore {
  const store = writable<TopBarMenuValue>(null);

  return {
    ...store,

    close() {
      store.set(null);
    },

    set(id) {
      store.set(id);
    },
  };
}

export default createTopBarMenuStore();
