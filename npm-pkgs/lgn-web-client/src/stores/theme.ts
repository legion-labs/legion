import type { Writable } from "svelte/store";

import { DefaultLocalStorage } from "../lib/storage";
import { connected } from "../lib/store";

export type ThemeValue = {
  name: string;
};

export type ThemeStore = Writable<ThemeValue>;

export const prefersColorSchemeThemeName = window.matchMedia(
  "(prefers-color-scheme: dark)"
).matches
  ? "dark"
  : "light";

export function createThemeStore(
  key: string,
  defaultThemeName: string = prefersColorSchemeThemeName
): ThemeStore {
  return connected<string, ThemeValue>(new DefaultLocalStorage(), key, {
    name: defaultThemeName,
  });
}
