import { planStructureCommand } from "../../organization/structure/commands/plan";
import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../../organization/structure/structure-projection";
import {
  assertStructureEditable,
  structureOperationRoots,
} from "../../organization/structure/structure-selection";
import type { CanvasNodeFrameUpdate, SpaceEditorDocument } from "../contract/canvas-document";
import { parseSpaceEditorDocument } from "../contract/canvas-document";
import {
  composeStructureTransforms,
  invertStructureTransform,
  normalizeStructureRotation,
  transformStructurePoint,
  IDENTITY_STRUCTURE_TRANSFORM,
} from "./structure-transform";

/** Transform gestures are converted once into local records, never into inferred membership. */
export function applyEditorFrames(
  document: SpaceEditorDocument,
  updates: readonly CanvasNodeFrameUpdate[],
): SpaceEditorDocument {
  const context = indexStructureSnapshot(document);
  const projection = projectStructure(context);
  const byId = new Map(updates.map((update) => [update.nodeId, update]));
  const roots = structureOperationRoots(context.tree, [...byId.keys()]);
  assertStructureEditable(
    context.tree,
    roots.map((item) => item.id),
  );
  let after = document;
  for (const item of roots) {
    const update = byId.get(item.id)!;
    const frame = update.frame;
    if (item.kind === "section") {
      if (frame.rotation !== 0) throw new Error("Sections cannot rotate");
      after = {
        ...after,
        ...planStructureCommand(after, {
          kind: "resize-section",
          id: item.id,
          bounds: { x: frame.x, y: frame.y, width: frame.width, height: frame.height },
        }).after,
      };
    } else if (item.kind === "group") {
      const bounds = structureBoundsInParent(context, projection, [item.id], item.id)!;
      const scale = frame.width / bounds.width;
      if (Math.abs(scale - frame.height / bounds.height) > 1e-6 * Math.max(1, scale))
        throw new Error("Groups support uniform scaling only");
      const transform = { x: 0, y: 0, rotation: normalizeStructureRotation(frame.rotation), scale };
      const offset = transformStructurePoint(transform, bounds);
      after = {
        ...after,
        ...planStructureCommand(after, {
          kind: "transform-group",
          id: item.id,
          worldTransform: { ...transform, x: frame.x - offset.x, y: frame.y - offset.y },
        }).after,
      };
    } else {
      const world = projection.transforms.get(item.id)!;
      const parent =
        item.parentId === null ? IDENTITY_STRUCTURE_TRANSFORM : projection.transforms.get(item.parentId)!;
      const transform = composeStructureTransforms(invertStructureTransform(parent), {
        ...world,
        x: frame.x,
        y: frame.y,
        rotation: normalizeStructureRotation(frame.rotation),
      });
      after = {
        ...after,
        layout: {
          version: 1,
          items: after.layout.items.map((layout) =>
            layout.kind === "content" && layout.itemId === item.id
              ? {
                  ...layout,
                  transform,
                  size: { width: frame.width / world.scale, height: frame.height / world.scale },
                  sizeAuthority: update.sizeAuthority ?? layout.sizeAuthority,
                }
              : layout,
          ),
        },
      };
    }
  }
  return parseSpaceEditorDocument(after);
}
