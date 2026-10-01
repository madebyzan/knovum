import type { SpaceNodeResizeMode } from "../../node-size";
import { CANVAS_TYPOGRAPHY_SCALE_MAX, CANVAS_TYPOGRAPHY_SCALE_MIN } from "../../schema/document";
import type { CanvasSceneNode } from "../scene/canvas-scene";

export interface CanvasSelectionTransformPolicy {
  contentAspect?: {
    ratio: number;
    fixedVerticalChrome: number;
  };
  resizeMode: SpaceNodeResizeMode;
  maxHeight: number;
  maxWidth: number;
  minHeight: number;
  minWidth: number;
  proportionalScale?: { baseHeight: number; baseWidth: number; min: number; max: number };
  nodeIds: readonly string[];
}

const DISABLED_SIZE = {
  resizeMode: "fixed" as const,
  maxHeight: Number.POSITIVE_INFINITY,
  maxWidth: Number.POSITIVE_INFINITY,
  minHeight: 1,
  minWidth: 1,
};

export function projectSelectionTransformPolicy(
  nodes: readonly CanvasSceneNode[],
  selectedNodeIds: ReadonlySet<string>,
): CanvasSelectionTransformPolicy {
  const selected = nodes.filter(({ node }) => selectedNodeIds.has(node.id));
  if (selected.length !== 1 || !selected[0]?.interaction.movable) {
    return { ...DISABLED_SIZE, nodeIds: selected.map(({ node }) => node.id) };
  }
  const [node] = selected;
  if (!node) return { ...DISABLED_SIZE, nodeIds: [] };
  const transform = node.interaction.transform;
  const typographyScale = node.layout.appearance?.typographyScale ?? 1;
  return {
    ...(transform.contentAspect ? { contentAspect: transform.contentAspect } : {}),
    resizeMode: transform.resizeMode,
    maxHeight: transform.maxHeight,
    maxWidth: transform.maxWidth,
    minHeight: transform.minHeight,
    minWidth: transform.minWidth,
    ...(node.node.data.kind === "text"
      ? {
          proportionalScale: {
            baseHeight: node.layout.frame.height,
            baseWidth: node.layout.frame.width,
            min: CANVAS_TYPOGRAPHY_SCALE_MIN / typographyScale,
            max: CANVAS_TYPOGRAPHY_SCALE_MAX / typographyScale,
          },
        }
      : {}),
    nodeIds: [node.node.id],
  };
}
