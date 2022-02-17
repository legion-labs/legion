import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { grpc } from "@improbable-eng/grpc-web";
import { authClient } from "@lgn/web-client/src/lib/auth";

export function getRemoteHost() {
  const remoteHost =
    "analytics-nlb-cddd70eafd32d85b.elb.ca-central-1.amazonaws.com";
  return remoteHost;
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
    new GrpcWebImpl("http://" + getRemoteHost() + ":9090", options)
  );

  return client;
}
