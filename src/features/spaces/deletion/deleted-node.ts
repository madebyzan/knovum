import type { SpaceNodeKind } from "../model/nodes/node-payload";

export interface SpaceDeletedNode {
  id: string;
  spaceId: string;
  spaceTitle: string;
  kind: SpaceNodeKind;
  title: string;
  deletedAt: number;
}
