import type { CanvasNodeAppearance, CanvasNodeFrameUpdate } from "../../schema/document";

export interface CanvasGestureUpdate extends CanvasNodeFrameUpdate {
  appearance?: CanvasNodeAppearance | null;
}
