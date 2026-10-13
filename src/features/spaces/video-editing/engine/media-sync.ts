/** Hard seeks flush the decoder, so playback only ever seeks when correction cannot catch up. */
export const HARD_SEEK_MS = 300;
const PAUSED_TOLERANCE_MS = 4;
const RATE_GAIN = 0.0016;
const MAX_RATE_TRIM = 0.06;

/**
 * Playback rate that closes a drift smoothly. A negative drift means the element is behind the
 * clock, so it plays slightly faster; the trim stays small enough to be inaudible.
 */
export function playbackRateFor(driftMs: number) {
  const trim = Math.max(-MAX_RATE_TRIM, Math.min(MAX_RATE_TRIM, -driftMs * RATE_GAIN));
  return Number((1 + trim).toFixed(4));
}

export type MediaSyncResult = "playing" | "waiting" | "parked";

export interface MediaTarget {
  sourceMs: number;
  playing: boolean;
}

/**
 * Aligns one media element with the timeline clock. While playing, the element keeps decoding and
 * only its rate is trimmed; a seek happens on scrub, on pre-roll or when drift is beyond repair.
 */
export function syncMedia(element: HTMLMediaElement, target: MediaTarget): MediaSyncResult {
  if (element.readyState < HTMLMediaElement.HAVE_METADATA) return "waiting";
  const expected = target.sourceMs / 1000;
  const driftMs = (element.currentTime - expected) * 1000;
  return target.playing ? follow(element, expected, driftMs) : park(element, expected, driftMs);
}

/** Scrub and pre-roll: the element is parked on an exact position with decoding stopped. */
function park(element: HTMLMediaElement, expected: number, driftMs: number): MediaSyncResult {
  if (!element.paused) element.pause();
  if (Math.abs(driftMs) > PAUSED_TOLERANCE_MS && !element.seeking) element.currentTime = expected;
  return element.seeking ? "waiting" : "parked";
}

/** Playback: the rate absorbs drift, and only an unrecoverable gap costs a seek. */
function follow(element: HTMLMediaElement, expected: number, driftMs: number): MediaSyncResult {
  if (Math.abs(driftMs) > HARD_SEEK_MS) {
    element.playbackRate = 1;
    if (!element.seeking) element.currentTime = expected;
    return "waiting";
  }
  const rate = playbackRateFor(driftMs);
  if (element.playbackRate !== rate) element.playbackRate = rate;
  if (element.seeking || element.readyState < HTMLMediaElement.HAVE_CURRENT_DATA) return "waiting";
  return "playing";
}

/** True once the element can paint the requested position. */
export function canPresent(element: HTMLMediaElement) {
  return !element.seeking && element.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA;
}
