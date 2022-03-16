import { negotiateLanguages } from "@fluent/langneg";
import { derived, get, Updater, Writable } from "svelte/store";
import { writable } from "svelte/store";
import type { AvailableLocalesStore, BundlesStore } from "./bundles";

export type LocaleValue = string;

export type LocaleStore = Writable<LocaleValue>;

/**
 * Stores the locale used by the user.
 * Uses the [Fluent algorithm](https://github.com/projectfluent/fluent.js/tree/master/fluent-langneg)
 * to decide which locale to use.
 *
 * Since the store is writable the locale can be set.
 * But _only known locales will be accepted_.
 * If the set locale is unknown, no changes are done.
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
  {
    defaultLocale,
    requestedLocales = navigator.languages,
  }: {
    defaultLocale: string;
    /** Locales that the user wants to use, defaults to `navigator.languages` */
    requestedLocales?: readonly string[];
  }
): LocaleStore {
  const exposedStore = derived(
    availableLocales,
    ($availableLocales) =>
      negotiateLanguages(requestedLocales, $availableLocales, {
        defaultLocale: defaultLocale,
        strategy: "lookup",
      })[0] || defaultLocale
  );

  const store = writable(get(exposedStore));

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
