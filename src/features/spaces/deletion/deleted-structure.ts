export interface SpaceDeletedStructureRoot {
  batchId: string;
  rootStructureNodeId: string;
  kind: "content" | "group" | "section";
  title: string;
  nodeId: string | null;
  deletedAt: number;
  spaceId: string;
  spaceTitle: string;
}
