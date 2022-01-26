import { GrpcWebImpl, PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";

export const performanceAnalyticsClient =  new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + location.hostname + ":9090", {})
  );