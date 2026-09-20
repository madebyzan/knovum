import { isEqual } from "es-toolkit";
import { z } from "zod";

import { spaceStructureSchema } from "../../model/structure/structure-schema";
import { validateStructureSnapshot } from "../../organization/structure/structure-snapshot-validation";
import { structureLayoutSchema } from "./structure-layout";
export {
  CANVAS_FILL_PRESETS,
  CANVAS_TEXT_COLORS,
  CANVAS_TYPOGRAPHY_SCALE_MIN,
  CANVAS_TYPOGRAPHY_SCALE_MAX,
  canvasNodeAppearanceSchema,
} from "./canvas-appearance";
export type { CanvasNodeAppearance } from "./canvas-appearance";
import type { CanvasNodeAppearance } from "./canvas-appearance";

export const MAX_CANVAS_Z_INDEX = 0xffff_ffff;
export const CANVAS_RELATION_COLORS = ["neutral", "blue", "green", "amber", "red", "purple"] as const;
export const CANVAS_RELATION_OVERRIDE_COLORS = ["blue", "green", "amber", "red", "purple"] as const;
export const canvasViewportSchema = z
  .object({ x: z.number().finite(), y: z.number().finite(), zoom: z.number().finite().positive() })
  .strict();
export const canvasRelationStyleSchema = z
  .object({ relationId: z.string().min(1).max(128), color: z.enum(CANVAS_RELATION_OVERRIDE_COLORS) })
  .strict();
export const spaceEditorDocumentSchema = z
  .object({
    version: z.literal(1),
    structure: spaceStructureSchema,
    layout: structureLayoutSchema,
    viewport: canvasViewportSchema,
    relationStyles: z.array(canvasRelationStyleSchema).max(100_000).optional(),
  })
  .strict()
  .superRefine((document, context) => {
    try {
      validateStructureSnapshot(document);
    } catch (error) {
      context.addIssue({ code: "custom", message: String(error), path: ["structure"] });
    }
    const identities = new Set<string>();
    for (const [index, style] of (document.relationStyles ?? []).entries()) {
      if (identities.has(style.relationId))
        context.addIssue({
          code: "custom",
          message: "Duplicate relation presentation identity",
          path: ["relationStyles", index],
        });
      identities.add(style.relationId);
    }
  });
export type SpaceEditorDocument = z.infer<typeof spaceEditorDocumentSchema>;
export type CanvasRelationColor = (typeof CANVAS_RELATION_COLORS)[number];
export type CanvasRelationOverrideColor = (typeof CANVAS_RELATION_OVERRIDE_COLORS)[number];
export type CanvasNodeSizeAuthority = "content" | "frame";
export interface CanvasNodeFrame {
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
}
/** Disposable world-space projection. This record is never persisted as organization state. */
export interface CanvasNodeLayoutV1 {
  nodeId: string;
  frame: CanvasNodeFrame;
  sizeAuthority: CanvasNodeSizeAuthority;
  zIndex: number;
  contentScale?: number | undefined;
  appearance?: CanvasNodeAppearance | undefined;
  fileSubpath?: string | undefined;
}
export interface CanvasNodeFrameUpdate {
  nodeId: string;
  frame: CanvasNodeFrame;
  sizeAuthority?: CanvasNodeSizeAuthority;
}
export function parseSpaceEditorDocument(value: unknown): SpaceEditorDocument {
  return spaceEditorDocumentSchema.parse(value);
}
export function emptySpaceEditorDocument(): SpaceEditorDocument {
  return {
    version: 1,
    structure: { version: 1, items: [] },
    layout: { version: 1, items: [] },
    viewport: { x: 0, y: 0, zoom: 1 },
  };
}
export function sameCanvasNodeAppearance(
  left: CanvasNodeAppearance | undefined,
  right: CanvasNodeAppearance | undefined,
) {
  return isEqual(left, right);
}
