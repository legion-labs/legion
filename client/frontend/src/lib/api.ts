export const stringToBytes = (s: string) => new TextEncoder().encode(s);

export const jsonToBytes = (j: Record<string, unknown>) =>
  stringToBytes(JSON.stringify(j));

export const bytesToString = (b: Uint8Array) => new TextDecoder().decode(b);

export const bytesToJson = <T>(b: Uint8Array): T =>
  JSON.parse(bytesToString(b));
