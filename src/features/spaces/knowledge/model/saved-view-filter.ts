import type { RelationKind } from "../../contracts/generated/RelationKind";
import type { RelationStatus } from "../../contracts/generated/RelationStatus";
import type { SpaceKnowledgeObjectStatus } from "./object-types";

export interface SpaceSavedViewFilter {
  text?: string | undefined;
  tags?: string[] | undefined;
  objectTypeIds?: string[] | undefined;
  statuses?: SpaceKnowledgeObjectStatus[] | undefined;
  relationKinds?: RelationKind[] | undefined;
  relationStatuses?: RelationStatus[] | undefined;
}
