export function jsonToBlob(j: Record<string, unknown>) {
  return new Blob([JSON.stringify(j)]);
}

export async function blobToJson<T>(b: Blob): Promise<T> {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return JSON.parse(await b.text());
}
