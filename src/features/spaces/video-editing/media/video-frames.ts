import type { CanvasSink, WrappedCanvas } from "mediabunny";

/** Frames held ahead of the playhead; the sink decodes further ahead on its own. */
const QUEUE_LIMIT = 3;
/** Tolerance behind the request before the window is considered stale and is re-seeked. */
const REWIND_S = 0.05;
/** Forward gap that is cheaper to seek to than to decode through. */
const JUMP_S = 0.75;
const JUMP_PLAYING_S = 2.0;

export interface FrameFeedEvents {
  onDecoded: () => void;
  onFailed: (error: unknown) => void;
}

/**
 * Presents the frame a timeline position needs from one video track. Decoding is pulled ahead
 * from the sink's sequential iterator, so frames are already queued when the playhead reaches them
 * and scrubbing restarts the iterator without decoding unwanted intermediate pictures.
 */
export function createFrameFeed(sink: CanvasSink, events: FrameFeedEvents) {
  let iterator: AsyncGenerator<WrappedCanvas, void, unknown> | undefined;
  let queue: WrappedCanvas[] = [];
  let current: WrappedCanvas | undefined;
  let previous: WrappedCanvas | undefined;
  let lastCanvas: CanvasImageSource | undefined;
  let generation = 0;
  let requesting = false;
  let exhausted = false;
  let startedS = 0;
  let decodedToS = 0;
  let advanced = false;
  let isPlaying = false;

  function restart(fromS: number) {
    generation += 1;
    void iterator?.return();
    iterator = sink.canvases(Math.max(0, fromS));
    queue = [];
    previous = current ?? previous;
    current = undefined;
    exhausted = false;
    requesting = false;
    startedS = fromS;
    decodedToS = fromS;
    advanced = false;
  }

  function stale(atS: number, playing: boolean) {
    if (!iterator) return true;
    if (requesting && decodedToS === startedS) return false;
    if (playing) {
      return atS > decodedToS + JUMP_PLAYING_S || atS < startedS - 0.2;
    }
    const floor = advanced && current ? Math.min(current.timestamp, atS) : startedS;
    return atS + REWIND_S < floor || atS > decodedToS + JUMP_S;
  }

  function accept(frame: WrappedCanvas) {
    queue.push(frame);
    decodedToS = frame.timestamp + frame.duration;
    events.onDecoded();
  }

  function request() {
    const active = iterator;
    if (!active || requesting || exhausted || queue.length >= QUEUE_LIMIT) return;
    const token = generation;
    requesting = true;
    void pull(active, token);
  }

  async function pull(active: AsyncGenerator<WrappedCanvas, void, unknown>, token: number) {
    try {
      const result = await active.next();
      if (token !== generation) return;
      requesting = false;
      if (result.done) exhausted = true;
      else accept(result.value);
    } catch (error) {
      if (token !== generation) return;
      requesting = false;
      exhausted = true;
      events.onFailed(error);
    }
  }

  function advance(atS: number) {
    let moved = false;
    while (queue.length > 0 && (!current || queue[0]!.timestamp <= atS)) {
      if (current) advanced = true;
      current = queue.shift();
      previous = undefined;
      moved = true;
    }
    if (moved && isPlaying) request();
  }

  return {
    /**
     * The newest decoded frame at or before the requested source position, or the first frame of a
     * fresh window while it is still catching up. Never returns a hole: until a frame arrives the
     * previous picture stays on screen.
     */
    frameAt(sourceS: number, playing: boolean) {
      isPlaying = playing;
      if (stale(sourceS, playing)) restart(sourceS);
      advance(sourceS);
      const covered =
        current &&
        (current.timestamp <= sourceS || !advanced) &&
        current.timestamp + current.duration > sourceS;
      if (playing || !covered) request();
      const frame = current ?? previous;
      if (frame?.canvas) lastCanvas = frame.canvas;
      return lastCanvas;
    },
    dispose() {
      generation += 1;
      void iterator?.return();
      iterator = undefined;
      queue = [];
      current = undefined;
      previous = undefined;
      lastCanvas = undefined;
    },
  };
}

export type FrameFeed = ReturnType<typeof createFrameFeed>;
