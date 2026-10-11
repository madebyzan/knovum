import type { RelationKind } from "../contracts/generated/RelationKind";
import type { RelationStatus } from "../contracts/generated/RelationStatus";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";

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
  relationKinds: RelationKind[];
  relationStatuses: RelationStatus[];
  queryFilter: SpaceSavedViewFilter;
  createdAt: number;
  updatedAt: number;
}
