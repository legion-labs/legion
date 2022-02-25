import { keepElementVisible } from "../lib/html";
import KeyboardNavigationStore from "../stores/keyboardNavigation";

export type Config = {
  size: number;
  store: KeyboardNavigationStore;
};

/**
 * Will "mark" the html element as a "navigable" item container.
 * By default the `keyboardNavigation` action will use the root element.
 */
export function keyboardNavigationContainer(htmlElement: HTMLElement) {
  htmlElement.dataset.keyboardNavigationContainer = "";
}

/** Will "mark" the html element as a "navigable" item */
export function keyboardNavigationItem(
  htmlElement: HTMLElement,
  index: number
) {
  htmlElement.dataset.keyboardNavigationItemIndex = index.toString();
}

export default function keyboardNavigation(
  htmlElement: HTMLElement,
  { size, store }: Config
) {
  htmlElement.tabIndex = -1;
  htmlElement.style.outline = "none";

  function handleKeyboard(event: KeyboardEvent) {
    store.update(({ currentIndex }) => {
      let newIndex: number | null = null;

      switch (event.key) {
        case "Enter": {
          htmlElement.dispatchEvent(
            new CustomEvent("navigation-select", { detail: currentIndex })
          );

          break;
        }

        case "F2": {
          htmlElement.dispatchEvent(
            new CustomEvent("navigation-rename", { detail: currentIndex })
          );

          break;
        }

        case "Delete": {
          htmlElement.dispatchEvent(
            new CustomEvent("navigation-remove", { detail: currentIndex })
          );

          break;
        }

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

      const container = htmlElement.querySelector(
        `[data-keyboard-navigation-container]`
      );

      // Auto scroll to "follow" the user focus when using the arrow keys
      keepElementVisible(container || htmlElement, element);

      htmlElement.dispatchEvent(
        new CustomEvent("navigation-change", { detail: newIndex })
      );

      return { currentIndex: newIndex };
    });
  }

  htmlElement.addEventListener("keydown", handleKeyboard);

  return {
    update({ size: newSize }: Config) {
      size = newSize;
    },
    destroy() {
      htmlElement.removeEventListener("keydown", handleKeyboard);
    },
  };
}
