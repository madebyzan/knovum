import type { ImageEditTool } from "../../image-editing/image-edit-session-store";

export const IMAGE_EDIT_TOOL_ORDER = [
  "adjust",
  "crop",
  "transform",
] as const satisfies readonly ImageEditTool[];

export type CanvasBarAction =
  | "node-primary"
  | "video-edit"
  | "node-speech"
  | "node-transcription"
  | "image-edit"
  | "image-remove-background"
  | "image-split-layers"
  | "image-remove-object";

/** Presentation-only overflow order; execution availability stays with the owning capability. */
const MINIMAL_MENU_ACTIONS: ReadonlySet<CanvasBarAction> = new Set([
  "node-primary",
  "video-edit",
  "node-speech",
  "node-transcription",
  "image-edit",
  "image-remove-background",
  "image-split-layers",
  "image-remove-object",
]);

export function canvasBarPlacement(action: CanvasBarAction, minimal: boolean): "bar" | "menu" {
  return minimal && MINIMAL_MENU_ACTIONS.has(action) ? "menu" : "bar";
}
