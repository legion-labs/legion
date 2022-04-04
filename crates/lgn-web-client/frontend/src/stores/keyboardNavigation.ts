import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export type KeyboardNavigationValue = {
  currentIndex: number | null;
};

export type KeyboardNavigationStore = Writable<KeyboardNavigationValue>;

export function createKeyboardNavigationStore(): KeyboardNavigationStore {
  return writable<KeyboardNavigationValue>({ currentIndex: null });
}
