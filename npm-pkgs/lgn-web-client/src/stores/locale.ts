import { negotiateLanguages } from "@fluent/langneg";
import type { Updater, Writable } from "svelte/store";
import { writable } from "svelte/store";
import { derived, get } from "svelte/store";

import type { Storage } from "../lib/storage";
import { connected } from "../lib/store";
import type { AvailableLocalesStore } from "./bundles";

export type LocaleValue = string;

export type LocaleStore = Writable<LocaleValue>;

export type LocalConfig = {
  /** Locales that the user wants to use, defaults to `navigator.languages` */
  requestedLocales?: readonly string[];
  /** By providing this option the locale will be "connected" to the provided storage */
  connect?: { key: string; storage: Storage<string, string> };
};

/**
 * Stores the locale used by the user.
 * Uses the [Fluent algorithm](https://github.com/projectfluent/fluent.js/tree/master/fluent-langneg)
 * to decide which locale to use.
 *
 * Since the store is writable the locale can be set.
 * But _only known locales will be accepted_.
 * If the set locale is unknown, no changes are done.
 *
 * This store can also be connected to a `Storage` like the local storage.
 *
 * If you don't want to rely on any sort of algorithm,
 * you can create a simple store on your end like follows:
 *
 * ```ts
 * const localeStore = writable("xx-XX");
 * ```
 */
export function createLocaleStore(
  availableLocales: AvailableLocalesStore,
  defaultLocale: string,
  { requestedLocales = navigator.languages, connect }: LocalConfig = {}
): LocaleStore {
  const exposedStore = derived(
    availableLocales,
    ($availableLocales) =>
      negotiateLanguages(requestedLocales, $availableLocales, {
        defaultLocale: defaultLocale,
        strategy: "lookup",
      })[0] || defaultLocale
  );

  let store: Writable<string>;

  if (connect) {
    store = connected(connect.storage, connect.key, get(exposedStore));
  } else {
    store = writable(get(exposedStore));
  }

  return {
    ...store,
    set(locale) {
      if (get(availableLocales).includes(locale)) {
        store.set(locale);
      }
    },
    update(f: Updater<string>) {
      store.update((locale) => {
        const desiredLocale = f(locale);

        if (!get(availableLocales).includes(locale)) {
          return locale;
        }

        return desiredLocale;
      });
    },
  };
}
