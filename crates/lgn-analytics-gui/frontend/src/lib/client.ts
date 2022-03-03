import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { grpc } from "@improbable-eng/grpc-web";
import { authClient } from "@lgn/web-client/src/lib/auth";

export function getRemoteHost(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST as string;
}

export function getUrl(): string {
  return import.meta.env.VITE_LEGION_ANALYTICS_API_URL as string;
}

export async function makeGrpcClient() {
  const metadata = new grpc.Metadata();
  const token = authClient.accessToken;

  if (!token) {
    throw new Error("Access token not found");
  }

  metadata.set("Authorization", "Bearer " + token);

  const options = { metadata: metadata };
  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl(getUrl(), options)
  );

  return client;
}
