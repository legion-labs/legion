import { FluentBundle, FluentResource } from "@fluent/bundle";
export type {
  TranslateValue,
  TranslateStore,
} from "@lgn/web-client/src/stores/translate";
import { createTranslateStore } from "@lgn/web-client/src/stores/translate";
export type {
  BundlesValue,
  BundlesStore,
  AvailableLocalesValue,
  AvailableLocalesStore,
} from "@lgn/web-client/src/stores/bundles";
import {
  createAvailableLocalesStore,
  createBundlesStore,
} from "@lgn/web-client/src/stores/bundles";
export type {
  LocaleValue,
  LocaleStore,
} from "@lgn/web-client/src/stores/locale";
import { createLocaleStore } from "@lgn/web-client/src/stores/locale";
import en from "@/assets/locales/en-US/example.ftl?raw";
import fr from "@/assets/locales/fr-CA/example.ftl?raw";

export const bundles = createBundlesStore();

// TODO: Automatically import from folder and import only the required locale
const enBundle = new FluentBundle(["en-US", "en"]);

enBundle.addResource(new FluentResource(en));

for (const locale of enBundle.locales) {
  bundles.add(locale, enBundle);
}

const frBundle = new FluentBundle(["fr-CA", "fr"]);

frBundle.addResource(new FluentResource(fr));

for (const locale of frBundle.locales) {
  bundles.add(locale, frBundle);
}

export const availableLocales = createAvailableLocalesStore(bundles);

export const locale = createLocaleStore(availableLocales, {
  defaultLocale: "en-US",
});

export const t = createTranslateStore(locale, bundles);
