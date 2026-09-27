import { z } from "zod";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { CanvasViewport } from "../contracts/canvas-editor";

const canvasViewStateSchema = z
  .object({
    x: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    y: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    zoom: z.number().finite().min(0.05).max(16),
  })
  .strict();

export const canvasViewStateOperations = {
  async load(spaceId: string): Promise<CanvasViewport | null> {
    const source = await invokeCommand<unknown>("plugin:spaces|load_canvas_view_state", { spaceId });
    return source === null ? null : canvasViewStateSchema.parse(source);
  },
  save(spaceId: string, view: CanvasViewport): Promise<void> {
    return invokeCommand("plugin:spaces|save_canvas_view_state", {
      request: { spaceId, view: canvasViewStateSchema.parse(view) },
    });
  },
};
