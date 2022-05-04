import type { FluentFunction, TextTransform } from "@fluent/bundle";
import { FluentBundle, FluentResource } from "@fluent/bundle";

import {
  createAvailableLocalesStore,
  createBundlesStore,
} from "../stores/bundles";
import type { AvailableLocalesStore, BundlesStore } from "../stores/bundles";
import type { LocalConfig, LocaleStore } from "../stores/locale";
import { createLocaleStore } from "../stores/locale";
import type { TranslateStore } from "../stores/translate";
import { createTranslateStore } from "../stores/translate";
import type { FluentBase } from "../types/fluent";

export type L10nOrchestrator<Fluent extends FluentBase> = {
  bundles: BundlesStore;
  availableLocales: AvailableLocalesStore;
  locale: LocaleStore;
  t: TranslateStore<Fluent>;
};

export type Locales = {
  names: string[];
  contents: string[];
};

export type L10nConfig = {
  local?: LocalConfig & {
    /** Passed down to the `FluentBundle` constructor */
    functions?: Record<string, FluentFunction>;
    /** Passed down to the `FluentBundle` constructor (defaults to `true`) */
    useIsolating?: boolean;
    /** Passed down to the `FluentBundle` constructor */
    transform?: TextTransform;
  };
};

/**
 * Keys of the `locales` argument are comma separated locales and values are
 * the contents of the locale themselves, typically imported from a fluent file like so:
 *
 * ```ts
 * import myLocale from "@/assets/locales/myLocale.ftl?raw";
 * ```
 */
export function createL10nOrchestrator<Fluent extends FluentBase>(
  locales: Locales[],
  config: L10nConfig = {}
): L10nOrchestrator<Fluent> {
  const bundles = createBundlesStore();

  for (const { names, contents } of locales) {
    const bundle = new FluentBundle(names, {
      functions: config.local?.functions,
      useIsolating: config.local?.useIsolating,
      transform: config.local?.transform,
    });

    for (const content of contents) {
      bundle.addResource(new FluentResource(content));
    }

    for (const locale of bundle.locales) {
      bundles.add(locale, bundle);
    }
  }

  const availableLocales = createAvailableLocalesStore(bundles);

  const locale = createLocaleStore(availableLocales, "en-US", config.local);

  const t = createTranslateStore<Fluent>(locale, bundles);

  return { bundles, availableLocales, locale, t };
}
