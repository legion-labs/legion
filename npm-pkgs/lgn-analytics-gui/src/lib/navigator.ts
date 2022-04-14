// Reexports svelte-navigator's components with a proper type
import type { SvelteComponentTyped } from "svelte";
import {
  Route as NavigatorRoute,
  Router as NavigatorRouter,
} from "svelte-navigator";
import type { RouteProps } from "svelte-navigator/types/Route";
import type { RouterProps } from "svelte-navigator/types/Router";

declare class InnerRouterProxy extends SvelteComponentTyped<RouterProps> {}

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
export const Router: typeof InnerRouterProxy = NavigatorRouter as any;

declare class InnerRouteProxy extends SvelteComponentTyped<RouteProps> {}

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
export const Route: typeof InnerRouteProxy = NavigatorRoute as any;
