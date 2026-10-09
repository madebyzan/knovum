import type { ImageEditTool } from "../../image-editing/image-edit-session-store";

export const IMAGE_EDIT_TOOL_ORDER = [
  "adjust",
  "crop",
  "transform",
] as const satisfies readonly ImageEditTool[];

export type CanvasBarAction =
  | "node-primary"
  | "video-edit"
  | "image-edit"
  | "image-remove-background"
  | "image-remove-object";

/** Presentation-only overflow order; execution availability stays with the owning capability. */
const MINIMAL_MENU_ACTIONS: ReadonlySet<CanvasBarAction> = new Set([
  "node-primary",
  "video-edit",
  "image-edit",
  "image-remove-background",
  "image-remove-object",
]);

export function canvasBarPlacement(action: CanvasBarAction, minimal: boolean): "bar" | "menu" {
  return minimal && MINIMAL_MENU_ACTIONS.has(action) ? "menu" : "bar";
}
