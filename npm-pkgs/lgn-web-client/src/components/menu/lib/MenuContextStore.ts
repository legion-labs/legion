import { writable } from "svelte/store";
import type { MenuItemDescription } from "./MenuItemDescription";

export type MenuContextStore = ReturnType<typeof getMenuContextStore>;

// export type MenuContext = {
//   open: boolean;
//   current?: MenuItemDescription | null;
// };

export function getMenuContextStore() {
  const { subscribe, set } = writable<MenuItemDescription | null>(null);

  // const updateState = (action: (state: MenuItemDescription | null) => void) => {
  //   update((s) => {
  //     action(s);

  //     return s;
  //   });
  // };

  const onClick = (item: MenuItemDescription) => {
    set(item);
  };

  const mouseEnter = (item: MenuItemDescription) => {
    set(item);
  };

  const close = () => {
    set(null);
  };

  return { subscribe, mouseEnter, onClick, close };
}
