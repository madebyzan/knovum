import type { ImageEditTool } from "../../image-editing/image-edit-session";

export const IMAGE_EDIT_TOOL_ORDER = [
  "adjust",
  "crop",
  "transform",
  "eraser",
] as const satisfies readonly ImageEditTool[];

export type CanvasBarAction = "node-primary" | "video-edit" | "image-edit" | "image-remove-background";

/** Presentation-only overflow order; execution availability stays with the owning capability. */
const MINIMAL_MENU_ACTIONS: ReadonlySet<CanvasBarAction> = new Set([
  "node-primary",
  "video-edit",
  "image-edit",
  "image-remove-background",
]);

export function canvasBarPlacement(action: CanvasBarAction, minimal: boolean): "bar" | "menu" {
  return minimal && MINIMAL_MENU_ACTIONS.has(action) ? "menu" : "bar";
}
