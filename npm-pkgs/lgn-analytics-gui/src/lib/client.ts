import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { enhanceGrpcClient } from "@lgn/web-client/src/lib/client";

import { accessTokenCookieName } from "@/constants";

export function createGrpcClient(url: string) {
  const rpc = new GrpcWebImpl(url, {});

  return enhanceGrpcClient(
    new PerformanceAnalyticsClientImpl(rpc),
    accessTokenCookieName
  );
}
