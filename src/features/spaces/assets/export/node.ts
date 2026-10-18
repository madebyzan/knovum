import { save } from "@tauri-apps/plugin-dialog";
import { extname } from "pathe";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { exportResource, exportTextFile } from "./ipc";
import { exportFilter, safeExportFileName } from "./naming";
import { payloadExports } from "./payloads";

export interface NodeExportOptions {
  /** Called once work that can be cancelled begins. */
  onStart?: (jobId: string) => void;
}

/**
 * Saves one resource in the format that was already chosen for it.
 *
 * The format is the instruction, so the destination only names the file: a
 * destination that does not carry the chosen extension is given it.
 */
export async function exportSpaceNode(
  spaceId: string,
  node: SpaceCanvasNode,
  format: string,
  options: NodeExportOptions = {},
): Promise<boolean> {
  const destination = await save({
    defaultPath: `${safeExportFileName(node.data.title) || node.data.kind}.${format}`,
    filters: [exportFilter(format)],
  });
  if (!destination) return false;
  const named = extname(destination).slice(1).toLowerCase() === format;
  const path = named ? destination : `${destination}.${format}`;
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
