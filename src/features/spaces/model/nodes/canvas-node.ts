import type { SpaceNodePayload } from "./node-payload";

interface CanvasNodeFor<T extends SpaceNodePayload> {
  id: string;
  type: T["kind"];
  data: T;
  initialPosition: { x: number; y: number };
  hidden?: boolean;
}

export type SpaceCanvasNode<T extends SpaceNodePayload = SpaceNodePayload> = T extends SpaceNodePayload
  ? CanvasNodeFor<T>
  : never;

export interface PositionedSpaceNode<T extends SpaceNodePayload = SpaceNodePayload> {
  node: SpaceCanvasNode<T>;
  position: { x: number; y: number };
}
