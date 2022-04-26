import type { Writable } from "svelte/store";

import { DefaultLocalStorage } from "../lib/storage";
import { connected } from "../lib/store";

export type ThemeValue = {
  name: string;
};

export type ThemeStore = Writable<ThemeValue>;

export const themeName = window.matchMedia("(prefers-color-scheme: dark)")
  .matches
  ? "dark"
  : "light";

export function createThemeStore(
  key: string,
  defaultThemeName: string = themeName
): ThemeStore {
  return connected<string, ThemeValue>(new DefaultLocalStorage(), key, {
    name: defaultThemeName,
  });
}
