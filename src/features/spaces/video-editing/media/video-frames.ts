import type { CanvasSink, WrappedCanvas } from "mediabunny";

/** Frames held ahead of the playhead; the sink decodes further ahead on its own. */
const QUEUE_LIMIT = 3;
/** Tolerance behind the request before the window is considered stale and is re-seeked. */
const REWIND_S = 0.02;
/** Forward gap that is cheaper to seek to than to decode through. */
const JUMP_S = 0.75;

export interface FrameFeedEvents {
  onDecoded: () => void;
  onFailed: (error: unknown) => void;
}

/**
 * Presents the frame a timeline position needs from one video track. Decoding is pulled one frame
 * at a time from the sink's sequential iterator, so work stays bounded per animation frame and a
 * backwards scrub or a long jump restarts the iterator instead of decoding everything in between.
 */
export function createFrameFeed(sink: CanvasSink, events: FrameFeedEvents) {
  let iterator: AsyncGenerator<WrappedCanvas, void, unknown> | undefined;
  let queue: WrappedCanvas[] = [];
  let current: WrappedCanvas | undefined;
  let generation = 0;
  let requesting = false;
  let exhausted = false;
  let startedS = 0;
  let decodedToS = 0;

  function restart(fromS: number) {
    generation += 1;
    void iterator?.return();
    iterator = sink.canvases(Math.max(0, fromS));
    queue = [];
    current = undefined;
    exhausted = false;
    requesting = false;
    startedS = fromS;
    decodedToS = fromS;
  }

  function stale(atS: number) {
    if (!iterator) return true;
    const floor = current?.timestamp ?? startedS;
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
    while (queue.length > 0 && (!current || queue[0]!.timestamp <= atS)) current = queue.shift();
  }

  return {
    /**
     * The newest decoded frame at or before the requested source position, or the first frame of a
     * fresh window while it is still catching up. Never returns a hole: until a frame arrives the
     * previous picture stays on screen.
     */
    frameAt(sourceS: number, playing: boolean) {
      if (stale(sourceS)) restart(sourceS);
      advance(sourceS);
      const covered = current && current.timestamp + current.duration > sourceS;
      if (playing || !covered) request();
      return current?.canvas;
    },
    dispose() {
      generation += 1;
      void iterator?.return();
      iterator = undefined;
      queue = [];
      current = undefined;
    },
  };
}

export type FrameFeed = ReturnType<typeof createFrameFeed>;
