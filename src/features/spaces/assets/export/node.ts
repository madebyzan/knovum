import { save } from "@tauri-apps/plugin-dialog";
import { extname } from "pathe";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { exportResource, exportTextFile, listExportFormats } from "./ipc";
import { exportFilter, safeExportFileName } from "./naming";
import { payloadExports } from "./payloads";

export interface NodeExportOptions {
  /** Called once a format is chosen and work that can be cancelled begins. */
  onStart?: (jobId: string) => void;
}

/**
 * Saves one resource in the format its destination names.
 *
 * A converted document, picture or recording carries the same content as its
 * source, so it belongs to the file the user is saving rather than to a second
 * resource on the canvas. The chosen extension is therefore the whole format
 * instruction, and the offered formats come from whoever owns the bytes.
 */
export async function exportSpaceNode(
  spaceId: string,
  node: SpaceCanvasNode,
  options: NodeExportOptions = {},
): Promise<boolean> {
  const payloads = payloadExports(node.data);
  const formats = [
    ...payloads.map(({ extension }) => extension),
    ...(await listExportFormats(spaceId, node.id)),
  ];
  const [fallback] = formats;
  if (!fallback) throw new Error("This resource has no exportable file");
  const destination = await save({
    defaultPath: `${safeExportFileName(node.data.title) || node.data.kind}.${fallback}`,
    filters: formats.map(exportFilter),
  });
  if (!destination) return false;
  // Dialogs that let the name be typed may leave the format unstated.
  const path = extname(destination) ? destination : `${destination}.${fallback}`;
  const payload = payloads.find(({ extension }) => extension === extname(path).slice(1).toLowerCase());
  if (payload) {
    await exportTextFile(path, payload.text());
    return true;
  }
  const jobId = crypto.randomUUID();
  options.onStart?.(jobId);
  await exportResource(spaceId, node.id, path, jobId);
  return true;
}
