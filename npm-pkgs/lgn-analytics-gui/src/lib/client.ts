import { grpc } from "@improbable-eng/grpc-web";

import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";

export function getRemoteHost(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST as string;
}

export function getUrl(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_API_URL as string;
}

export function makeGrpcClient(accessToken: string | null) {
  if (accessToken === null) {
    return new PerformanceAnalyticsClientImpl(new GrpcWebImpl(getUrl(), {}));
  }

  const metadata = new grpc.Metadata();

  metadata.set("Authorization", `Bearer ${accessToken}`);

  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl(getUrl(), { metadata })
  );

  return client;
}
