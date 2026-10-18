import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { listExportFormats } from "./ipc";
import { payloadExports } from "./payloads";

/**
 * The formats one resource admits, its own format first.
 *
 * A converted document, picture or recording carries the same content as its
 * source, so it belongs to the file being saved rather than to a second
 * resource on the canvas. Which formats exist is answered by whoever owns the
 * bytes: the Renderer for content it already holds, the engine for the rest.
 */
export async function nodeExportFormats(spaceId: string, node: SpaceCanvasNode): Promise<readonly string[]> {
  return [
    ...payloadExports(node.data).map(({ extension }) => extension),
    ...(await listExportFormats(spaceId, node.id)),
  ];
}

/** A section is an arrangement, so its only format is the outline of what it holds. */
export const SECTION_EXPORT_FORMATS: readonly string[] = ["json"];
