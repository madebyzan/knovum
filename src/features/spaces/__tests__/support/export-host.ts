import { vi } from "vitest";

import { createSpaceNode } from "../../canvas/node";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

/** The Tauri surface an export touches: engine commands and the save dialog. */
export const exportHost = {
  invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
};

export const tauriCore = { invoke: exportHost.invoke };
export const tauriDialog = { save: exportHost.save };

export function canvasNode(nodeId: string, data: SpaceCanvasNode["data"]): SpaceCanvasNode {
  return createSpaceNode(data, { x: 0, y: 0 }, nodeId);
}
