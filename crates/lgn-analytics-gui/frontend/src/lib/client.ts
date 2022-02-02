import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";

export const client = new PerformanceAnalyticsClientImpl(
  new GrpcWebImpl("http://" + location.hostname + ":9090", {})
);
