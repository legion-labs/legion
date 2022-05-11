import { writable } from "svelte/store";

// Not meant to be used as is in production
// as the menu might become dynamic at one point
export type Id = typeof menus[number]["id"];

export type TopBarMenuValue = Id | null;

export type TopBarMenuStore = ReturnType<typeof createTopBarMenuStore>;

export const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Window" },
  { id: 4, title: "Help" },
] as const;

function createTopBarMenuStore() {
  const store = writable<TopBarMenuValue>(null);

  return {
    ...store,

    close() {
      store.set(null);
    },

    set(id: Id) {
      store.set(id);
    },
  };
}

export default createTopBarMenuStore();
