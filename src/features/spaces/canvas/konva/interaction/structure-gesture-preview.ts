import type { SpaceEditCommand } from "../../../contracts/generated/SpaceEditCommand";
import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import type { StructureLayout, LocalTransform } from "../../../model/structure/schema/structure";
import { atLeastSectionMinimum, sectionMinimumScale } from "../../../model/structure/section-metrics";
import { createStructureIndex } from "../../../model/structure/structure-index";
import { relativeTransform, worldTransformFor } from "../../../model/structure/world-transform";
import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";

export type SectionResizeCorner = "north-west" | "north-east" | "south-west" | "south-east";
export type StructureGestureCommand = Extract<
  SpaceEditCommand,
  { kind: "structure.move" | "section.resize-frame" | "structure.transform" }
>;

/** Ephemeral layout only. Rust remains the authority for persisted edits and membership. */
export function projectStructureGesture(
  snapshot: SpaceEditSnapshot,
  command: StructureGestureCommand,
): SpaceEditSnapshot {
  if (command.kind === "section.resize-frame") return resizePreview(snapshot, command);
  const index = createStructureIndex(snapshot.structure);
  const layoutByStructureNodeId = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const roots = new Set(command.rootIds);
  return {
    ...snapshot,
    layouts: snapshot.layouts.map((layout) => {
      if (!roots.has(layout.structureNodeId)) return layout;
      if (layout.kind === "section") {
        if (command.kind !== "structure.move") throw new Error("Sections cannot rotate or scale");
        return {
          ...layout,
          position: { x: layout.position.x + command.delta.x, y: layout.position.y + command.delta.y },
        };
      }
      const parentId = index.structureById.get(layout.structureNodeId)!.parentId;
      const parent = parentId
        ? worldTransformFor(parentId, index, layoutByStructureNodeId)
        : { x: 0, y: 0, rotation: 0, scale: 1 };
      const world = worldTransformFor(layout.structureNodeId, index, layoutByStructureNodeId);
      const next =
        command.kind === "structure.move"
          ? { ...world, x: world.x + command.delta.x, y: world.y + command.delta.y }
          : transformWorld(world, command);
      return { ...layout, transform: relativeTransform(next, parent) };
    }),
  };
}

function transformWorld(
  world: LocalTransform,
  command: Extract<StructureGestureCommand, { kind: "structure.transform" }>,
): LocalTransform {
  const radians = (command.rotationDelta * Math.PI) / 180;
  const x = world.x - command.anchor.x;
  const y = world.y - command.anchor.y;
  return {
    x:
      command.anchor.x +
      command.scaleFactor * (x * Math.cos(radians) - y * Math.sin(radians)) +
      command.translation.x,
    y:
      command.anchor.y +
      command.scaleFactor * (x * Math.sin(radians) + y * Math.cos(radians)) +
      command.translation.y,
    rotation: world.rotation + command.rotationDelta,
    scale: world.scale * command.scaleFactor,
  };
}

function resizePreview(
  snapshot: SpaceEditSnapshot,
  command: Extract<StructureGestureCommand, { kind: "section.resize-frame" }>,
): SpaceEditSnapshot {
  const section = snapshot.layouts.find((layout) => layout.structureNodeId === command.structureNodeId);
  if (section?.kind !== "section") throw new Error("Section layout is unavailable");
  const delta = { x: section.position.x - command.frame.x, y: section.position.y - command.frame.y };
  const children = new Set(
    snapshot.structure.filter(({ parentId }) => parentId === command.structureNodeId).map(({ id }) => id),
  );
  return {
    ...snapshot,
    layouts: snapshot.layouts.map((layout): StructureLayout => {
      if (layout.structureNodeId === command.structureNodeId)
        return {
          ...section,
          position: { x: command.frame.x, y: command.frame.y },
          size: { width: command.frame.width, height: command.frame.height },
        };
      if (!children.has(layout.structureNodeId)) return layout;
      if (layout.kind === "section") throw new Error("Sections cannot nest");
      return {
        ...layout,
        transform: { ...layout.transform, x: layout.transform.x + delta.x, y: layout.transform.y + delta.y },
      };
    }),
  };
}

/**
 * The frame a corner drag asks for; the opposite corner stays put. With the aspect locked the frame
 * scales uniformly, following whichever axis the pointer has pulled further.
 */
export function resizedSectionFrame(
  frame: CanvasRect,
  corner: SectionResizeCorner,
  delta: CanvasPoint,
  aspectLocked: boolean,
): CanvasRect {
  const west = corner.endsWith("west");
  const north = corner.startsWith("north");
  const dragged = {
    width: frame.width + (west ? -delta.x : delta.x),
    height: frame.height + (north ? -delta.y : delta.y),
  };
  // A pointer pulled past the opposite corner gives a negative scale; the minimum floors it.
  const scale = Math.max(
    sectionMinimumScale(frame),
    dragged.width / frame.width,
    dragged.height / frame.height,
  );
  const { width, height } = aspectLocked
    ? { width: frame.width * scale, height: frame.height * scale }
    : atLeastSectionMinimum(dragged, false);
  return {
    x: west ? frame.x + frame.width - width : frame.x,
    y: north ? frame.y + frame.height - height : frame.y,
    width,
    height,
  };
}

export function structureGestureChangesLayout(snapshot: SpaceEditSnapshot, command: StructureGestureCommand) {
  if (command.kind === "structure.move") return command.delta.x !== 0 || command.delta.y !== 0;
  if (command.kind === "structure.transform")
    return (
      command.rotationDelta !== 0 ||
      command.scaleFactor !== 1 ||
      command.translation.x !== 0 ||
      command.translation.y !== 0
    );
  return sectionFrameChanges(snapshot, command);
}

function sectionFrameChanges(
  snapshot: SpaceEditSnapshot,
  command: Extract<StructureGestureCommand, { kind: "section.resize-frame" }>,
) {
  const layout = snapshot.layouts.find(({ structureNodeId }) => structureNodeId === command.structureNodeId);
  if (layout?.kind !== "section") throw new Error("Section layout is unavailable");
  const { frame } = command;
  return (
    layout.position.x !== frame.x ||
    layout.position.y !== frame.y ||
    layout.size.width !== frame.width ||
    layout.size.height !== frame.height
  );
}
