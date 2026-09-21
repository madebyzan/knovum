import type { SpaceSavedViewFilter } from "../knowledge/model/projections";
import type { SpaceRelationKind, SpaceRelationStatus } from "../model/relations/relation";

export interface SpaceScene {
  id: string;
  spaceId: string;
  title: string;
  thumbnailPath: string | null;
  viewportX: number;
  viewportY: number;
  viewportZoom: number;
  orderIndex: number;
  focusStructureNodeIds: string[];
  relationKinds: SpaceRelationKind[];
  relationStatuses: SpaceRelationStatus[];
  queryFilter: SpaceSavedViewFilter;
  createdAt: number;
  updatedAt: number;
}
