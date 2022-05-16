import { writable } from "svelte/store";
import type { MenuItemDescription } from "./MenuItemDescription";

export type MenuContextStore = ReturnType<typeof getMenuContextStore>;

export type MenuContext = {
  open: boolean;
  current?: MenuItemDescription | null;
};

export function getMenuContextStore() {
  const { subscribe, update } = writable<MenuContext>({ open: false });

  const updateState = (action: (state: MenuContext) => void) => {
    update((s) => {
      action(s);

      return s;
    });
  };

  const onRootClick = (item: MenuItemDescription) => {
    updateState((s) => {
      s.open = true;
      s.current = item;
    });
  };

  const mouseEnter = (item: MenuItemDescription) => {
    updateState((s) => {
      if (s.open) {
        s.current = item;
      }
    });
  };

  const close = () => {
    updateState((s) => {
      s.open = false;
      s.current = null;
    });
  };

  return { subscribe, mouseEnter, onRootClick, close };
}
