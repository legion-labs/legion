import { grpc } from "@improbable-eng/grpc-web";
import { derived } from "svelte/store";
import type { Readable } from "svelte/store";

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

export function createGrpcClientStore(
  accessTokenStore: Readable<string | null>
) {
  return derived(accessTokenStore, ($accessToken) => {
    if ($accessToken === null) {
      return new PerformanceAnalyticsClientImpl(new GrpcWebImpl(getUrl(), {}));
    }

    const metadata = new grpc.Metadata();

    metadata.set("Authorization", `Bearer ${$accessToken}`);

    const client = new PerformanceAnalyticsClientImpl(
      new GrpcWebImpl(getUrl(), { metadata })
    );

    return client;
  });
}
