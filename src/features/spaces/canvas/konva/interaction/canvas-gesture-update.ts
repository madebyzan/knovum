import type { CanvasNodeAppearance, CanvasNodeFrameUpdate } from "../../contract/canvas-document";

export interface CanvasGestureUpdate extends CanvasNodeFrameUpdate {
  contentScale?: number;
  appearance?: CanvasNodeAppearance | null;
}
