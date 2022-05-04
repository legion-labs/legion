import type { FluentBundle, FluentVariable } from "@fluent/bundle";
import type { Readable } from "svelte/store";
import { derived } from "svelte/store";

import { displayError } from "../lib/errors";
import log from "../lib/log";
import type { BundlesStore } from "./bundles";
import type { LocaleStore } from "./locale";

export type TranslateValue = (
  id: string,
  args?: Record<string, FluentVariable> | null
) => string;

export type TranslateStore = Readable<TranslateValue>;

// TODO: Add errors support
function translate(
  locale: string,
  bundles: Map<string, FluentBundle>,
  id: string,
  args?: Record<string, FluentVariable> | null
) {
  const errors: Error[] = [];

  const bundle = bundles.get(locale);

  if (!bundle) {
    return "";
  }

  const message = bundle.getMessage(id);

  const translatedMessage = message?.value
    ? bundle.formatPattern(message.value, args, errors)
    : "";

  if (errors.length) {
    log.error(
      log.json`Couldn't translate message ${id} (${args}): ${displayError(
        errors.map((error) => displayError(error)).join(", ")
      )}`
    );
  }

  return translatedMessage;
}

export function createTranslateStore(
  locale: LocaleStore,
  bundles: BundlesStore
): TranslateStore {
  return derived([locale, bundles], ([$locale, $bundles]) =>
    translate.bind(null, $locale, $bundles)
  );
}
