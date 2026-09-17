import type { CanvasNodeAppearance, CanvasNodeFrameUpdate } from "../../contract/canvas-document";

export interface CanvasGestureUpdate extends CanvasNodeFrameUpdate {
  appearance?: CanvasNodeAppearance | null;
}
