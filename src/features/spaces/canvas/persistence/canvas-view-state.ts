import { z } from "zod";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { CanvasViewport } from "../contracts/canvas-editor-port";

const canvasViewStateSchema = z
  .object({
    x: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    y: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    zoom: z.number().finite().min(0.05).max(16),
  })
  .strict();

/**
 * The camera of every Space this window has shown. The window is the only writer of view state, so
 * what it last read or wrote is the stored camera: a canvas returning to a Space starts from it
 * without a round-trip, and writes reach the database in the order they were made.
 */
const views = new Map<string, CanvasViewport | null>();
let writes = Promise.resolve();

export const canvasViewStateOperations = {
  async load(spaceId: string): Promise<CanvasViewport | null> {
    if (views.has(spaceId)) return views.get(spaceId)!;
    const source = await invokeCommand<unknown>("plugin:spaces|load_canvas_view_state", { spaceId });
    // A camera written while this read was in flight is newer than what the read returned.
    if (!views.has(spaceId)) views.set(spaceId, source === null ? null : canvasViewStateSchema.parse(source));
    return views.get(spaceId)!;
  },
  save(spaceId: string, view: CanvasViewport): Promise<void> {
    const parsed = canvasViewStateSchema.parse(view);
    views.set(spaceId, parsed);
    const write = writes.then(() =>
      invokeCommand<void>("plugin:spaces|save_canvas_view_state", { request: { spaceId, view: parsed } }),
    );
    // A failed write is reported to its caller and must not hold back the writes after it.
    writes = write.catch(() => undefined);
    return write;
  },
  /** Resolves once every write made so far has reached the database or failed to its caller. */
  settled(): Promise<void> {
    return writes;
  },
};
