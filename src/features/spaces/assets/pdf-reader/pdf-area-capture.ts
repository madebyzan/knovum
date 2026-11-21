import { addPassageNode } from "../../canvas/passage-capture";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";
import { buildSourceReaderLocator } from "../../reading/source-reader-target";
import { persistSpaceImageBytes } from "../asset-operations";
import type { PdfReaderSession } from "./pdf-editor-params";

/** Renders a PDF rectangular crop at Retina DPI and adds it as a Canvas Image Node with provenance. */
export async function capturePdfAreaToCanvas(
  session: PdfReaderSession,
  sourceNode: SpaceCanvasNode,
  page: number,
  rect: PdfNormalizedRect,
): Promise<SpaceCanvasNode | undefined> {
  const rendered = await session.renderRegionPng(page, rect);
  const persisted = await persistSpaceImageBytes(rendered.bytes);
  const title = `${sourceNode.data.title} · p.${page}`;
  const payload: SpaceImagePayload = {
    kind: "image",
    title,
    fileName: `pdf-crop-p${page}.png`,
    assetPath: persisted.path,
    sourceRevision: persisted.sourceRevision,
    intrinsicWidth: rendered.width,
    intrinsicHeight: rendered.height,
  };
  const quote = `PDF · p.${page}`;
  const locator = buildSourceReaderLocator(sourceNode.data, { kind: "pdf", page, rects: [rect] }, quote);
  return addPassageNode(payload, [{ nodeId: sourceNode.id, locator, quote }]);
}
