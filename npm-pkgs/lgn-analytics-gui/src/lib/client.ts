import { grpc } from "@improbable-eng/grpc-web";

import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { authClient } from "@lgn/web-client/src/lib/auth";

export function getRemoteHost(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST as string;
}

export function getUrl(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_API_URL as string;
}

export function makeGrpcClient() {
  const metadata = new grpc.Metadata();
  const token = authClient.accessToken;

  if (!token) {
    return new PerformanceAnalyticsClientImpl(new GrpcWebImpl(getUrl(), {}));
  }

  metadata.set("Authorization", "Bearer " + token);

  const options = { metadata: metadata };
  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl(getUrl(), options)
  );

  return client;
}
