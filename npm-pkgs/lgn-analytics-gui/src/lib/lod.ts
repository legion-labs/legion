export function getLod(minMs: number, maxMs: number, pixelWidth: number) {
  const deltaMs = maxMs - minMs;
  const pixelSizeMs = deltaMs / pixelWidth;
  return getLodFromPixelSizeMs(pixelSizeMs);
}

export function getLodFromPixelSizeMs(pixelSizeMs: number) {
  return getLodFromPixelSizeNs(pixelSizeMs * 1_000_000);
}

export function getLodFromPixelSizeNs(pixelSizeNs: number) {
  return Math.max(0, Math.floor(Math.log(pixelSizeNs) / Math.log(100)));
}

export function MergeThresholdForLOD(lod: number): number {
  return Math.pow(100, lod - 2) / 10;
}
