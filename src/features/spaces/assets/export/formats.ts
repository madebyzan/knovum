import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { listExportFormats } from "./ipc";
import { payloadExports } from "./payloads";

/** What a selection can be saved as, and why anything further is unavailable. */
export interface ExportFormats {
  /** The admitted formats, in order; the first is the selection's own format. */
  formats: readonly string[];
  /** Why nothing further is offered, when the content is outside a contract. */
  unavailable: string | null;
}

/**
 * The formats one resource admits, its own format first.
 *
 * A converted document, picture or recording carries the same content as its
 * source, so it belongs to the file being saved rather than to a second
 * resource on the canvas. Which formats exist is answered by whoever owns the
 * bytes: the Renderer for content it already holds, the engine for the rest.
 * A format both of them can write is offered once, by the Renderer, because
 * that is the resource's own serialization. Content the engine cannot read
 * admits nothing and says why.
 */
export async function nodeExportFormats(spaceId: string, node: SpaceCanvasNode): Promise<ExportFormats> {
  const engine = await listExportFormats(spaceId, node.id);
  return {
    formats: [
      ...new Set([...payloadExports(node.data).map(({ extension }) => extension), ...engine.formats]),
    ],
    unavailable: engine.unavailable,
  };
}

/** A section is an arrangement, so its only format is the outline of what it holds. */
export const SECTION_EXPORT_FORMATS: ExportFormats = { formats: ["json"], unavailable: null };
