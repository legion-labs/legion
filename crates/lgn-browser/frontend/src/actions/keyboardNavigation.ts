import { keepElementVisible } from "../lib/html";
import { Writable } from "svelte/store";

export type Store = {
  currentIndex: number | null;
};

export type Config = {
  disabled: boolean;
  listener?(index: number): void;
  size: number;
  store: Writable<Store>;
};

/** Will "mark" the html element as a "navigable" item */
export function keyboardNavigationItem(
  htmlElement: HTMLElement,
  index: number
) {
  htmlElement.dataset.keyboardNavigationItemIndex = index.toString();
}

export default function keyboardNavigation(
  htmlElement: HTMLElement,
  { disabled, listener, size, store }: Config
) {
  function handleWindowKeyword(event: KeyboardEvent) {
    if (disabled) {
      return null;
    }

    store.update(({ currentIndex }) => {
      let newIndex: number | null = null;

      switch (event.key) {
        case "ArrowUp": {
          // `currentIndex` should never be lt 0
          newIndex =
            currentIndex === null || currentIndex <= 0
              ? size - 1
              : currentIndex - 1;

          break;
        }

        case "ArrowDown": {
          // currentIndex should never be gt `items.length - 1`
          newIndex =
            currentIndex === null || currentIndex >= size - 1
              ? 0
              : currentIndex + 1;

          break;
        }
      }

      if (newIndex == null) {
        return { currentIndex };
      }

      event.preventDefault();

      const element = htmlElement.querySelector(
        `[data-keyboard-navigation-item-index="${newIndex}"]`
      );

      if (!element) {
        return { currentIndex };
      }

      // Auto scroll to "follow" the user focus when using the arrow keys
      keepElementVisible(htmlElement, element);

      listener && listener(newIndex);

      return { currentIndex: newIndex };
    });
  }

  window.addEventListener("keydown", handleWindowKeyword);

  return {
    update({ disabled: newDisabled, size: newSize }: Config) {
      disabled = newDisabled;
      size = newSize;
    },
    destroy() {
      window.removeEventListener("keydown", handleWindowKeyword);
    },
  };
}
