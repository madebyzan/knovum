import type { SpaceNodePayload } from "../model/nodes/node-payload";

export interface SpaceClipboardNode {
  id: string;
  position: { x: number; y: number };
  data: SpaceNodePayload;
}

export interface SpaceClipboardPayload {
  version: 1;
  nodes: SpaceClipboardNode[];
}
