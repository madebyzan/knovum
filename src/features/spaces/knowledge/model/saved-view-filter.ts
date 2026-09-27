import type { SpaceRelationKind, SpaceRelationStatus } from "../../model/relations/relation";
import type { SpaceKnowledgeObjectStatus } from "./object-types";

export interface SpaceSavedViewFilter {
  text?: string | undefined;
  tags?: string[] | undefined;
  objectTypeIds?: string[] | undefined;
  statuses?: SpaceKnowledgeObjectStatus[] | undefined;
  relationKinds?: SpaceRelationKind[] | undefined;
  relationStatuses?: SpaceRelationStatus[] | undefined;
}
