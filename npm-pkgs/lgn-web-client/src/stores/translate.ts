import type { FluentBundle } from "@fluent/bundle";
import type { Readable } from "svelte/store";
import { derived } from "svelte/store";

import { displayError } from "../lib/errors";
import log from "../lib/log";
import type {
  FluentBaseVariablesOnly,
  ResolveFluentArgumentsVariablesOnly,
} from "../types/fluent";
import type { BundlesStore } from "./bundles";
import type { LocaleStore } from "./locale";

export type TranslateValue<Fluent extends FluentBaseVariablesOnly> = <
  Id extends keyof Fluent
>(
  ...args: ResolveFluentArgumentsVariablesOnly<Fluent, Id>
) => string;

export type TranslateStore<Fluent extends FluentBaseVariablesOnly> = Readable<
  TranslateValue<Fluent>
>;

// TODO: Add errors support
function translate<
  Fluent extends FluentBaseVariablesOnly,
  Id extends keyof Fluent
>(
  locale: string,
  bundles: Map<string, FluentBundle>,
  ...[id, variables]: ResolveFluentArgumentsVariablesOnly<Fluent, Id>
) {
  const errors: Error[] = [];

  const bundle = bundles.get(locale);

  if (!bundle) {
    return "";
  }

  const message = bundle.getMessage(id as string);

  const translatedMessage = message?.value
    ? bundle.formatPattern(message.value, variables, errors)
    : "";

  if (errors.length) {
    log.error(
      log.json`Couldn't translate message ${id} (${variables}): ${displayError(
        errors.map((error) => displayError(error)).join(", ")
      )}`
    );
  }

  return translatedMessage;
}

export function createTranslateStore<Fluent extends FluentBaseVariablesOnly>(
  locale: LocaleStore,
  bundles: BundlesStore
): TranslateStore<Fluent> {
  return derived(
    [locale, bundles],
    ([$locale, $bundles]) =>
      function <Id extends keyof Fluent>(
        ...args: ResolveFluentArgumentsVariablesOnly<Fluent, Id>
      ) {
        return translate<Fluent, Id>($locale, $bundles, ...args);
      }
  );
}
