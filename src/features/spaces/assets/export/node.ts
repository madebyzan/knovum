import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { chooseExportDestination } from "./destination";
import { renderNodeMermaidDiagrams } from "./diagrams";
import { exportResource, exportTextFile } from "./ipc";
import { safeExportFileName } from "./naming";
import { serializePayloadExport } from "./payloads";

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
  const text = await serializePayloadExport(node, format);
  if (text !== undefined) {
    await exportTextFile(path, text);
    return true;
  }
  const renderedDiagrams = await renderNodeMermaidDiagrams(node.data, format);
  const jobId = crypto.randomUUID();
  options.onStart?.(jobId);
  await exportResource({
    spaceId,
    nodeId: node.id,
    destinationPath: path,
    jobId,
    renderedDiagrams,
  });
  return true;
}
