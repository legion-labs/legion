import { Process } from "@lgn/proto-telemetry/dist/process";
import { getLodFromPixelSizeMs } from "./lod";
import { ThreadBlock, ThreadBlockLOD } from "./Timeline/ThreadBlock";

export function timestampToMs(process: Process, timestamp: number): number {
  const nbTicks = timestamp - process.startTicks;
  return (nbTicks * 1000.0) / process.tscFrequency;
}

export function processMsOffsetToRoot(
  currentProcess: Process | undefined,
  process: Process
): number {
  if (!currentProcess?.startTime) {
    throw new Error("Parent process start time undefined");
  }
  const parentStartTime = Date.parse(currentProcess?.startTime);
  return Date.parse(process.startTime) - parentStartTime;
}

export function computePreferredBlockLod(
  canvasWidth: number,
  viewRange: [number, number],
  block: ThreadBlock
): number | null {
  const beginBlock = block.beginMs;
  const endBlock = block.endMs;
  return computePreferredLodFromTimeRange(
    canvasWidth,
    viewRange,
    beginBlock,
    endBlock
  );
}

function computePreferredLodFromTimeRange(
  canvasWidth: number,
  vr: [number, number],
  beginMs: number,
  endMs: number
): number | null {
  if (beginMs > vr[1] || endMs < vr[0]) {
    return null;
  }
  const currentPixelSize = (vr[1] - vr[0]) / canvasWidth;
  return getLodFromPixelSizeMs(currentPixelSize);
}

export function findBestLod(
  canvasWidth: number,
  vr: [number, number],
  block: ThreadBlock
): ThreadBlockLOD | null {
  const preferredLod = computePreferredLodFromTimeRange(
    canvasWidth,
    vr,
    block.beginMs,
    block.endMs
  );
  if (preferredLod == null) {
    return null;
  }
  return block.lods.reduce((lhs, rhs) => {
    if (lhs.tracks.length == 0) {
      return rhs;
    }
    if (rhs.tracks.length == 0) {
      return lhs;
    }
    if (
      Math.abs(lhs.lodId - preferredLod) < Math.abs(rhs.lodId - preferredLod)
    ) {
      return lhs;
    } else {
      return rhs;
    }
  });
}
