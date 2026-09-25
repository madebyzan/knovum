import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import { spaceNodeResizePolicy, type SpaceNodeResizeMode } from "./node-size";

export interface CanvasNodeInteraction {
  activationMode: "activate-to-interact" | "primary" | "select-to-interact";
  connectable: boolean;
  movable: boolean;
  transform: {
    resizeMode: SpaceNodeResizeMode;
    maxHeight: number;
    maxWidth: number;
    minHeight: number;
    minWidth: number;
    contentAspect?: {
      ratio: number;
      fixedVerticalChrome: number;
    };
  };
}

const NON_CONNECTABLE_KINDS = new Set<SpaceNodeKind>(["credential", "portal"]);
const ACTIVATE_TO_INTERACT_KINDS = new Set<SpaceNodeKind>(["code", "note", "text", "website"]);
const SELECT_TO_INTERACT_KINDS = new Set<SpaceNodeKind>(["audio", "credential", "video"]);

export function isCanvasNodeConnectable(node: SpaceCanvasNode) {
  return !NON_CONNECTABLE_KINDS.has(node.data.kind);
}

export function canvasNodeInteraction(node: SpaceCanvasNode, locked = false): CanvasNodeInteraction {
  const resize = spaceNodeResizePolicy(node.data);
  const textLayout = node.data.kind === "text";
  const movable = !locked;
  return {
    activationMode: nodeActivationMode(node),
    connectable: isCanvasNodeConnectable(node),
    movable,
    transform: {
      ...resize,
      minHeight: textLayout ? 1 : resize.minHeight,
      maxHeight: textLayout ? Number.POSITIVE_INFINITY : resize.maxHeight,
      resizeMode: movable ? resize.resizeMode : "fixed",
    },
  };
}

function nodeActivationMode(node: SpaceCanvasNode): CanvasNodeInteraction["activationMode"] {
  if (SELECT_TO_INTERACT_KINDS.has(node.data.kind)) return "select-to-interact";
  if (node.data.kind === "file" && (node.data.entries?.length ?? 0) > 0) return "select-to-interact";
  if (ACTIVATE_TO_INTERACT_KINDS.has(node.data.kind)) return "activate-to-interact";
  return "primary";
}
