import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { chooseExportDestination } from "./destination";
import { exportResource, exportTextFile } from "./ipc";
import { safeExportFileName } from "./naming";
import { payloadExports } from "./payloads";

export interface NodeExportOptions {
  /** Called once work that can be cancelled begins. */
  onStart?: (jobId: string) => void;
}

/** Saves one resource in the format that was already chosen for it. */
export async function exportSpaceNode(
  spaceId: string,
  node: SpaceCanvasNode,
  format: string,
  options: NodeExportOptions = {},
): Promise<boolean> {
  const path = await chooseExportDestination(safeExportFileName(node.data.title) || node.data.kind, format);
  if (!path) return false;
  const payload = payloadExports(node.data).find(({ extension }) => extension === format);
  if (payload) {
    await exportTextFile(path, payload.text());
    return true;
  }
  const jobId = crypto.randomUUID();
  options.onStart?.(jobId);
  await exportResource(spaceId, node.id, path, jobId);
  return true;
}
