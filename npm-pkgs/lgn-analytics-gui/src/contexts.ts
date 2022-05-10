/** This module exposes short functions that retrieve a well typed version of the values in contexts */
import { getContext } from "svelte";
import type { Writable } from "svelte/store";

import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import type { L10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";
import type { NotificationsStore } from "@lgn/web-client/src/stores/notifications";
import type { ThemeStore } from "@lgn/web-client/src/stores/theme";

import {
  debugContextKey,
  httpClientContextKey,
  l10nOrchestratorContextKey,
  notificationsContextKey,
  themeContextKey,
  threadItemLengthContextKey,
} from "./constants";

/** Get the theme context */
export function getThemeContext() {
  return getContext<ThemeStore>(themeContextKey);
}

/** Get the thread item CSS with in pixel context */
export function getThreadItemLengthContext() {
  return getContext<number>(threadItemLengthContextKey);
}

/** Get the l10n store set context */
export function getL10nOrchestratorContext() {
  return getContext<L10nOrchestrator<Fluent>>(l10nOrchestratorContextKey);
}

/** Get the http client context */
export function getHttpClientContext() {
  return getContext<PerformanceAnalyticsClientImpl>(httpClientContextKey);
}

/** Get the notifications context */
export function getNotificationsContext() {
  return getContext<NotificationsStore>(notificationsContextKey);
}

/** Get the debug context */
export function getDebugContext() {
  return getContext<Writable<boolean>>(debugContextKey);
}
