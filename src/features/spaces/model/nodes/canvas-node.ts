import type { SpaceNodeKind, SpaceNodePayload } from "./node-payload";

export interface SpaceCanvasNode<T extends SpaceNodePayload = SpaceNodePayload> {
  id: string;
  type: SpaceNodeKind;
  data: T;
  initialPosition: { x: number; y: number };
  hidden?: boolean;
}

export interface PositionedSpaceNode<T extends SpaceNodePayload = SpaceNodePayload> {
  node: SpaceCanvasNode<T>;
  position: { x: number; y: number };
}
