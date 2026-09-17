import type { SpaceRelationKind, SpaceRelationStatus } from "../../model/relations/relation";
import type { SpaceKnowledgeObjectStatus } from "./object-types";

export interface SpaceSavedViewFilter {
  text?: string;
  tags?: string[];
  objectTypeIds?: string[];
  statuses?: SpaceKnowledgeObjectStatus[];
  relationKinds?: SpaceRelationKind[];
  relationStatuses?: SpaceRelationStatus[];
}
