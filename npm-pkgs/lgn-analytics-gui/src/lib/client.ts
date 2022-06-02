import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { enhanceGrpcClient } from "@lgn/web-client/src/lib/grpcClient";

import { accessTokenCookieName } from "@/constants";

export function getRemoteHost(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST as string;
}

export function getUrl(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_API_URL as string;
}

export function createGrpcClient() {
  const rpc = new GrpcWebImpl(getUrl(), {});

  return enhanceGrpcClient(
    new PerformanceAnalyticsClientImpl(rpc),
    accessTokenCookieName
  );
}
