import { writable } from "svelte/store";

export type Value = {
  currentIndex: number | null;
};

export function createKeyboardNavigationStore() {
  return writable<Value>({ currentIndex: null });
}
