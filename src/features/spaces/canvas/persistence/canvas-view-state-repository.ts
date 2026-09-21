import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type { CanvasViewport } from "../contract/canvas-editor";

const canvasViewStateSchema = z
  .object({
    x: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    y: z.number().finite().min(-1_000_000_000).max(1_000_000_000),
    zoom: z.number().finite().min(0.05).max(16),
  })
  .strict();

export const canvasViewStateRepository = {
  async load(spaceId: string): Promise<CanvasViewport | null> {
    const source = await invoke<unknown>("plugin:spaces|load_canvas_view_state", { spaceId });
    return source === null ? null : canvasViewStateSchema.parse(source);
  },
  save(spaceId: string, view: CanvasViewport): Promise<void> {
    return invoke("plugin:spaces|save_canvas_view_state", {
      request: { spaceId, view: canvasViewStateSchema.parse(view) },
    });
  },
};
