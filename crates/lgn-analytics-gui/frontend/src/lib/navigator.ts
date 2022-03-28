// Reexports svelte-navigator's components with a proper type

import type { RouterProps } from "svelte-navigator/types/Router";
import type { RouteProps } from "svelte-navigator/types/Route";
import {
  Router as NavigatorRouter,
  Route as NavigatorRoute,
} from "svelte-navigator";

import type { SvelteComponentTyped } from "svelte";

declare class InnerRouterProxy extends SvelteComponentTyped<RouterProps> {}

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
export const Router: typeof InnerRouterProxy = NavigatorRouter as any;

declare class InnerRouteProxy extends SvelteComponentTyped<RouteProps> {}

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
export const Route: typeof InnerRouteProxy = NavigatorRoute as any;
