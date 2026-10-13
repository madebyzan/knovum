/** Drift that the playback rate can no longer absorb, so the element is repositioned once. */
export const HARD_SEEK_MS = 300;
/** Drift tolerated while parked; below it a seek would cost more than it corrects. */
const PARKED_TOLERANCE_MS = 20;
const RATE_GAIN = 0.0016;
const MAX_RATE_TRIM = 0.06;
/** `HTMLMediaElement.readyState` steps, named rather than read off the constructor. */
const HAVE_METADATA = 1;
const HAVE_CURRENT_DATA = 2;

export type MediaSyncResult = "waiting" | "parked" | "playing";

/**
 * Playback rate that walks an element back to the timeline clock. Correcting drift by rate instead
 * of by seeking is what keeps a cut continuous: a seek drops the decoded frame and shows a gap.
 */
export function playbackRateFor(driftMs: number) {
  const trim = Math.max(-MAX_RATE_TRIM, Math.min(MAX_RATE_TRIM, -driftMs * RATE_GAIN));
  return Number((1 + trim).toFixed(4));
}

export function canPresent(element: HTMLMediaElement) {
  return !element.seeking && element.readyState >= HAVE_CURRENT_DATA;
}

/** Signed distance between where the element stands and where the clock wants it. */
function offsetMs(element: HTMLMediaElement, sourceMs: number) {
  return element.currentTime * 1000 - sourceMs;
}

/**
 * Holds a paused element on an exact position. The seek is issued once; while the browser is
 * seeking the element is left alone, because re-issuing `currentTime` restarts the decode and is
 * what made scrubbing flicker.
 */
export function syncParked(element: HTMLMediaElement, sourceMs: number): MediaSyncResult {
  if (element.readyState < HAVE_METADATA) return "waiting";
  if (element.seeking) return "waiting";
  if (Math.abs(offsetMs(element, sourceMs)) > PARKED_TOLERANCE_MS) {
    element.currentTime = sourceMs / 1000;
    return "waiting";
  }
  return canPresent(element) ? "parked" : "waiting";
}

/**
 * Keeps a playing element locked to the clock. Only an unrecoverable gap costs a seek; everything
 * smaller is absorbed by a sub-audible rate trim.
 */
export function syncPlaying(element: HTMLMediaElement, sourceMs: number): MediaSyncResult {
  if (element.readyState < HAVE_METADATA) return "waiting";
  if (element.seeking) return "waiting";
  const drift = offsetMs(element, sourceMs);
  if (Math.abs(drift) > HARD_SEEK_MS) {
    element.playbackRate = 1;
    element.currentTime = sourceMs / 1000;
    return "waiting";
  }
  const rate = playbackRateFor(drift);
  if (element.playbackRate !== rate) element.playbackRate = rate;
  return canPresent(element) ? "playing" : "waiting";
}
