import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import { spaceNodeResizePolicy, type SpaceNodeResizeMode } from "../../node-size";
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
    contentAspect?: { ratio: number; fixedVerticalChrome: number };
  };
}
const SELECT = new Set(["audio", "credential", "video"]);
const ACTIVATE = new Set(["code", "note", "text", "website"]);
export function canvasNodeInteraction(node: CanvasDisplayNode, locked = false): CanvasNodeInteraction {
  const data = node.data;
  if (data.kind === "group" || data.kind === "section")
    return {
      activationMode: "primary",
      connectable: false,
      movable: !locked,
      transform: {
        resizeMode:
          locked || (data.kind === "section" && data.collapsed)
            ? "fixed"
            : data.kind === "group"
              ? "proportional"
              : "free",
        minWidth: data.kind === "section" ? 240 : 1,
        minHeight: data.kind === "section" ? 160 : 1,
        maxWidth: Infinity,
        maxHeight: Infinity,
      },
    };
  const resize = spaceNodeResizePolicy(data);
  const activationMode =
    SELECT.has(data.kind) || (data.kind === "file" && Boolean(data.entries?.length))
      ? "select-to-interact"
      : ACTIVATE.has(data.kind)
        ? "activate-to-interact"
        : "primary";
  return {
    activationMode,
    connectable: !["credential", "portal"].includes(data.kind),
    movable: !locked,
    transform: {
      ...resize,
      minHeight: data.kind === "text" ? 1 : resize.minHeight,
      resizeMode: locked ? "fixed" : resize.resizeMode,
    },
  };
}
