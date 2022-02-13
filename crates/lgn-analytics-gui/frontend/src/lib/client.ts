import {
  GrpcWebImpl,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { grpc } from "@improbable-eng/grpc-web";
import { getAccessToken } from "@lgn/web-client/src/stores/userInfo";

export async function makeGrpcClient() {
  let metadata = new grpc.Metadata();
  const token = await getAccessToken();
  metadata.set("Authorization", "Bearer " + token);
  const options = { metadata: metadata };
  const remoteHost = "analytics-nlb-cddd70eafd32d85b.elb.ca-central-1.amazonaws.com";
  const client = new PerformanceAnalyticsClientImpl(
    new GrpcWebImpl("http://" + remoteHost + ":9090", options)
  );
  return client;
}
