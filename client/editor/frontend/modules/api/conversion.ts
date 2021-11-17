// `atob` and `btoa` are indeed deprecated on Node but not (yet) on Browsers.
// See https://github.com/microsoft/TypeScript/issues/45566 for more.
// We use the `window.*` workaround for now and might move to Buffer using
// this polyfill https://github.com/feross/buffer later.

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function bytesOut(x: any): string {
  return window.btoa(JSON.stringify(x));
}

export function bytesIn(x: string): unknown {
  return JSON.parse(window.atob(x));
}
