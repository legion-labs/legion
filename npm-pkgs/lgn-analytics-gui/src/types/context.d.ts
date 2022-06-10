import type { Writable } from "svelte/store";

import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";
import type { L10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";
import type { NotificationsStore } from "@lgn/web-client/src/stores/notifications";
import type { ThemeStore } from "@lgn/web-client/src/stores/theme";

import type {
  MetricConfigStore,
  MetricStore,
  RecentlyUsedMetricStore,
  SelectedMetricStore,
} from "@/components/Metric/Lib/MetricStore";
import type { RuntimeConfig } from "@/lib/runtimeConfig";

declare module "svelte" {
  // Global
  function setContext(
    key: "runtime-config",
    context: RuntimeConfig
  ): RuntimeConfig;
  function setContext(key: "theme", context: ThemeStore): ThemeStore;
  function setContext(key: "thread-item-length", context: number): number;
  function setContext(
    key: typeof l10nOrchestratorContextKey,
    context: L10nOrchestrator<Fluent>
  ): L10nOrchestrator<Fluent>;
  function setContext(
    key: "http-client",
    context: PerformanceAnalyticsClientImpl
  ): PerformanceAnalyticsClientImpl;
  function setContext(
    key: "notifications",
    context: NotificationsStore<Fluent>
  ): NotificationsStore<Fluent>;
  function setContext(
    key: "debug",
    context: Writable<boolean>
  ): Writable<boolean>;

  // Metrics
  function setContext(key: "metrics-store", context: MetricStore): MetricStore;
  function setContext(
    key: "selected-metrics-store",
    context: SelectedMetricStore
  ): SelectedMetricStore;
  function setContext(
    key: "metrics-config-store",
    context: MetricConfigStore
  ): MetricConfigStore;
  function setContext(
    key: "recently-used-metrics-store",
    context: RecentlyUsedMetricStore
  ): RecentlyUsedMetricStore;

  // Add a new setContext function:
  // function setContext(key: "new-key", context: Value): Value;

  // Catch all setContext, keep this
  function setContext(key: string, context: unknown): never;
  function setContext<_>(key: string, context: unknown): never;

  // Global
  function getContext(key: "runtime-config"): RuntimeConfig;
  function getContext(key: "theme"): ThemeStore;
  function getContext(key: "thread-item-length"): number;
  function getContext(
    key: typeof l10nOrchestratorContextKey
  ): L10nOrchestrator<Fluent>;
  function getContext(key: "http-client"): PerformanceAnalyticsClientImpl;
  function getContext(key: "notifications"): NotificationsStore<Fluent>;
  function getContext(key: "debug"): Writable<boolean>;

  // Metrics
  function getContext(key: "metrics-store"): MetricStore;
  function getContext(key: "selected-metrics-store"): SelectedMetricStore;
  function getContext(key: "metrics-config-store"): MetricConfigStore;
  function getContext(
    key: "recently-used-metrics-store"
  ): RecentlyUsedMetricStore;

  // Add a new getContext function:
  // function getContext(key: "new-key"): Value;

  // Catch all getContext, keep this
  function getContext(key: string): never;
  function getContext<_>(key: string): never;
}
