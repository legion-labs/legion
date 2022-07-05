import { grpc } from "@improbable-eng/grpc-web";

import type { ApiClient } from "@lgn/api";

import { authClient } from "./auth";
import { getCookie } from "./cookie";
import log from "./log";

/**
 * Will create a Proxy around the provided GRPC client that will automatically refresh the token set
 * if a request is performed when the user is not authenticated.
 *
 * _Warning: It's not possible to properly type the `client` argument,
 * so pretty much anything will be accepted by this function, don't overuse it_
 */
export function enhanceGrpcClient<Client extends object>(
  client: Client,
  accessTokenCookieName: string,
  { minLatency = 5 }: { minLatency?: number } = {}
) {
  const state = {
    clientIsRefreshingToken: false,
  };

  return new Proxy(client, {
    get(target, propertyKey, receiver) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const f: ((...args: unknown[]) => Promise<unknown>) | undefined =
        Reflect.get(target, propertyKey, receiver);

      if (!f) {
        return;
      }

      return async function (
        ...[params, metadataArg]: [unknown, grpc.Metadata | undefined]
      ) {
        if (state.clientIsRefreshingToken) {
          await new Promise<void>((resolve) => {
            const id = setInterval(() => {
              if (!state.clientIsRefreshingToken) {
                clearInterval(id);
                resolve();
              }
            }, minLatency);
          });
        }

        let accessToken = getCookie(accessTokenCookieName);

        if (accessToken === null) {
          state.clientIsRefreshingToken = true;

          log.debug(
            "http-client",
            "Access token not found, trying to refresh the client token set"
          );

          try {
            const clientTokenSet = await authClient.refreshClientTokenSet();

            authClient.storeClientTokenSet(clientTokenSet);

            accessToken = clientTokenSet.access_token;

            state.clientIsRefreshingToken = false;
          } catch {
            log.debug(
              "http-client",
              "Couldn't refresh the client token set, redirecting to the idp"
            );

            state.clientIsRefreshingToken = false;

            const authorizationUrl = await authClient.getAuthorizationUrl();

            if (authorizationUrl !== null) {
              window.location.href = authorizationUrl;
            }

            return;
          }
        }

        const metadata = new grpc.Metadata();

        if (metadataArg) {
          metadata.forEach((key, values) => {
            metadata.set(key, values);
          });
        }

        metadata.set("Authorization", `Bearer ${accessToken}`);

        return f(params, metadata);
      };
    },
  });
}

// TODO: Drop and use version from @lgn/api
/**
 * Automatically appends the auth header to all requests, automatically refreshes the token set
 * if a request is performed when the user is not authenticated.
 */
export function addAuthToClient<Client extends ApiClient>(
  client: Client,
  _accessTokenCookieName: string,
  _: { minLatency?: number } = {}
): Client {
  return client;
  // const state = {
  //   clientIsRefreshingToken: false,
  // };

  // client.addRequestStartInterceptor(async (input, init) => {
  //   if (state.clientIsRefreshingToken) {
  //     await new Promise<void>((resolve) => {
  //       const id = setInterval(() => {
  //         if (!state.clientIsRefreshingToken) {
  //           clearInterval(id);
  //           resolve();
  //         }
  //       }, minLatency);
  //     });
  //   }

  //   let accessToken = getCookie(accessTokenCookieName);

  //   if (accessToken === null) {
  //     state.clientIsRefreshingToken = true;

  //     log.debug(
  //       "http-client",
  //       "Access token not found, trying to refresh the client token set"
  //     );

  //     try {
  //       const clientTokenSet = await authClient.refreshClientTokenSet();

  //       authClient.storeClientTokenSet(clientTokenSet);

  //       accessToken = clientTokenSet.access_token;

  //       state.clientIsRefreshingToken = false;
  //     } catch {
  //       log.debug(
  //         "http-client",
  //         "Couldn't refresh the client token set, redirecting to the idp"
  //       );

  //       state.clientIsRefreshingToken = false;

  //       const authorizationUrl = await authClient.getAuthorizationUrl();

  //       if (authorizationUrl !== null) {

  //         window.location.href = authorizationUrl;
  //       }

  //       return [input, init] as [RequestInfo | URL, RequestInit | undefined];
  //     }
  //   }

  //   if (input instanceof Request) {
  //     input.headers.set("Authorization", `Bearer ${accessToken}`);
  //   }

  //   if (init?.headers instanceof Headers) {
  //     init.headers.set("Authorization", `Bearer ${accessToken}`);
  //   }

  //   return [input, init] as [RequestInfo | URL, RequestInit | undefined];
  // });

  // return client;
}
