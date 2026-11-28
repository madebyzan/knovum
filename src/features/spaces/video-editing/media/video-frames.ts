/** Large jump/cut threshold where hardware playback snaps immediately to the new position. */
const HARD_SYNC_DRIFT_S = 0.5;
/** Smooth phase-lock band corrected by subtle playbackRate nudging with zero seek hiccup. */
const SOFT_SYNC_DRIFT_S = 0.06;
/** Minimum cooldown after a seek or play() startup so AVFoundation never gets trapped in a seek loop. */
const SEEK_COOLDOWN_MS = 500;
/** Seek tolerance while paused so identical frames do not re-trigger hardware seeks. */
const PAUSED_SEEK_EPSILON_S = 0.012;

export interface FrameFeedEvents {
  onDecoded: () => void;
  onFailed: (error: unknown) => void;
}

/**
 * Native hardware video feed backed directly by the platform media engine (AVFoundation /
 * VideoToolbox on macOS, Media Foundation on Windows) via an `HTMLVideoElement`.
 *
 * Play startup is issued once per playback segment and guarded by a seek cooldown so long-GOP
 * videos never get aborted mid-keyframe decode; drift within 0.5s is corrected silently via
 * playbackRate nudging so playback stays locked at the video's native frame rate.
 */
export function createFrameFeed(
  url: string,
  events: FrameFeedEvents,
  createElement: () => HTMLVideoElement = () => document.createElement("video"),
) {
  const video = createElement();
  video.muted = true;
  video.playsInline = true;
  video.preload = "auto";
  video.src = url;

  let disposed = false;
  let failed = false;
  let playInFlight = false;
  let lastSeekAtMs = 0;
  let pendingSeekS: number | undefined;
  let lastRequestedPauseS = -1;

  function clampTime(targetS: number): number {
    const clamped = Math.max(0, targetS);
    if (Number.isFinite(video.duration) && video.duration > 0) {
      return Math.min(clamped, Math.max(0, video.duration - 0.005));
    }
    return clamped;
  }

  function setRate(rate: number) {
    if (video.playbackRate !== rate) video.playbackRate = rate;
  }

  function applySeek(targetS: number) {
    const clamped = clampTime(targetS);
    if (video.readyState < HTMLMediaElement.HAVE_METADATA || video.seeking) {
      pendingSeekS = clamped;
      return;
    }
    if (Math.abs(video.currentTime - clamped) > PAUSED_SEEK_EPSILON_S) {
      lastSeekAtMs = performance.now();
      video.currentTime = clamped;
    }
  }

  function flushPendingSeek() {
    if (disposed || pendingSeekS === undefined || video.seeking) return;
    const next = pendingSeekS;
    pendingSeekS = undefined;
    applySeek(next);
  }

  function onLoadedData() {
    if (disposed) return;
    flushPendingSeek();
    events.onDecoded();
  }

  function onSeeked() {
    if (disposed) return;
    flushPendingSeek();
    events.onDecoded();
  }

  function onError() {
    if (disposed || failed) return;
    failed = true;
    playInFlight = false;
    events.onFailed(new Error(video.error?.message || "Video playback failed"));
  }

  video.addEventListener("loadedmetadata", flushPendingSeek);
  video.addEventListener("loadeddata", onLoadedData);
  video.addEventListener("seeked", onSeeked);
  video.addEventListener("error", onError);

  function startPlayback(target: number) {
    if (playInFlight) return;
    if (Math.abs(target - video.currentTime) > 0.08 && !video.seeking) {
      applySeek(target);
    }
    setRate(1);
    playInFlight = true;
    lastSeekAtMs = performance.now();
    const promise = video.play();
    if (promise) {
      void promise
        .catch(() => {})
        .finally(() => {
          playInFlight = false;
        });
    } else {
      playInFlight = false;
    }
  }

  function syncPlaying(sourceS: number) {
    lastRequestedPauseS = -1;
    const target = clampTime(sourceS);
    if (video.paused) {
      startPlayback(target);
      return;
    }
    if (video.seeking || video.readyState < HTMLMediaElement.HAVE_CURRENT_DATA) return;
    const drift = target - video.currentTime;
    const absDrift = Math.abs(drift);
    if (absDrift > HARD_SYNC_DRIFT_S && performance.now() - lastSeekAtMs >= SEEK_COOLDOWN_MS) {
      setRate(1);
      applySeek(target);
      return;
    }
    setRate(absDrift > SOFT_SYNC_DRIFT_S ? (drift > 0 ? 1.04 : 0.96) : 1);
  }

  function syncPaused(sourceS: number) {
    playInFlight = false;
    if (!video.paused) video.pause();
    setRate(1);
    if (Math.abs(sourceS - lastRequestedPauseS) <= 0.002) return;
    lastRequestedPauseS = sourceS;
    applySeek(sourceS);
  }

  return {
    frameAt(sourceS: number, playing: boolean): HTMLVideoElement | undefined {
      if (disposed || failed) return undefined;
      if (playing) syncPlaying(sourceS);
      else syncPaused(sourceS);
      return video;
    },
    prewarm(sourceS: number) {
      if (disposed || failed || !video.paused) return;
      applySeek(sourceS);
    },
    pause() {
      if (disposed) return;
      playInFlight = false;
      if (!video.paused) video.pause();
      setRate(1);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      playInFlight = false;
      pendingSeekS = undefined;
      video.removeEventListener("loadedmetadata", flushPendingSeek);
      video.removeEventListener("loadeddata", onLoadedData);
      video.removeEventListener("seeked", onSeeked);
      video.removeEventListener("error", onError);
      video.pause();
      video.removeAttribute("src");
      video.load();
    },
  };
}

export type FrameFeed = ReturnType<typeof createFrameFeed>;
