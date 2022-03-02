// See https://kit.svelte.dev/docs/hooks for more
// We use this file solely to turn our whole app into an SPA

import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) =>
  resolve(event, {
    ssr: false,
  });
