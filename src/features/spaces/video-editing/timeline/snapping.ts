import { editPoints } from "../edits/commit";
import type { VideoEditProject } from "../project";
import { snapToFrame } from "../timing";
import { pxToTime } from "./scale";

const SNAP_TOLERANCE_PX = 7;

export interface SnapResult {
  timeMs: number;
  snappedTo?: number | undefined;
}

/**
 * Snaps a dragged time to meaningful neighbours — clip and text edges plus the playhead — and
 * otherwise to the output frame grid, so an edit never lands between two frames by accident.
 */
export function snapTime(options: {
  project: VideoEditProject;
  timeMs: number;
  pxPerSecond: number;
  playheadMs?: number | undefined;
  disabled?: boolean | undefined;
  exclude?: readonly number[] | undefined;
}): SnapResult {
  const frameAligned = snapToFrame(options.project, options.timeMs);
  if (options.disabled) return { timeMs: Math.round(options.timeMs) };
  const tolerance = pxToTime(SNAP_TOLERANCE_PX, options.pxPerSecond);
  const excluded = new Set(options.exclude ?? []);
  const candidates = [
    ...editPoints(options.project),
    ...(options.playheadMs === undefined ? [] : [options.playheadMs]),
  ].filter((point) => !excluded.has(point));
  let best: { point: number; distance: number } | undefined;
  for (const point of candidates) {
    const distance = Math.abs(point - options.timeMs);
    if (distance <= tolerance && (!best || distance < best.distance)) best = { point, distance };
  }
  return best ? { timeMs: best.point, snappedTo: best.point } : { timeMs: Math.round(frameAligned) };
}
