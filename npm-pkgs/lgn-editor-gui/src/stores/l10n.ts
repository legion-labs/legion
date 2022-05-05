import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
import { createL10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";

import en from "@/assets/locales/en-US/example.ftl?raw";
import fr from "@/assets/locales/fr-CA/example.ftl?raw";
import { localeStorageKey } from "@/constants";

const l10n = createL10nOrchestrator<Fluent>(
  [
    {
      names: ["en-US", "en"],
      contents: [en],
    },
    {
      names: ["fr-CA", "fr"],
      contents: [fr],
    },
  ],
  {
    local: {
      connect: {
        key: localeStorageKey,
        storage: new DefaultLocalStorage(),
      },
    },
  }
);

export const { availableLocales, bundles, locale, t } = l10n;

export default l10n;
