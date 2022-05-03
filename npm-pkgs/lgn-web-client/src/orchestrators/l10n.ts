import { FluentBundle, FluentResource } from "@fluent/bundle";
import {
  createAvailableLocalesStore,
  createBundlesStore,
  type AvailableLocalesStore,
  type BundlesStore,
} from "../stores/bundles";
import type { LocalConfig, LocaleStore } from "../stores/locale";
import type { TranslateStore } from "../stores/translate";
import { createLocaleStore } from "../stores/locale";
import { createTranslateStore } from "../stores/translate";

export type L10nOrchestrator = {
  bundles: BundlesStore;
  availableLocales: AvailableLocalesStore;
  locale: LocaleStore;
  t: TranslateStore;
};

export type L10nConfig = {
  local?: LocalConfig;
};

/**
 * Keys of the `locales` argument are comma separated locales and values are
 * the contents of the locale themselves, typically imported from a fluent file like so:
 *
 * ```ts
 * import myLocale from "@/assets/locales/myLocale.ftl?raw";
 * ```
 */
export function createL10nOrchestrator(
  locales: Record<string, string[]>,
  config: L10nConfig = {}
): L10nOrchestrator {
  const bundles = createBundlesStore();

  for (const [name, contents] of Object.entries(locales)) {
    const bundle = new FluentBundle(name.split(",").map((name) => name.trim()));

    for (const content of contents) {
      bundle.addResource(new FluentResource(content));
    }

    for (const locale of bundle.locales) {
      bundles.add(locale, bundle);
    }
  }

  const availableLocales = createAvailableLocalesStore(bundles);

  const locale = createLocaleStore(availableLocales, "en-US", config.local);

  const t = createTranslateStore(locale, bundles);

  return { bundles, availableLocales, locale, t };
}
