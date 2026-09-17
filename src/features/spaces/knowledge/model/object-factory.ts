import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { SYSTEM_OBJECT_TYPES, type SystemKnowledgeObjectKind } from "./object-types";

export type { SystemKnowledgeObjectKind } from "./object-types";

export function createSystemKnowledgeObjectPayload(
  kind: SystemKnowledgeObjectKind,
  title: string,
): SpaceNodePayload {
  const objectType = SYSTEM_OBJECT_TYPES[kind];
  return {
    kind: "object",
    objectTypeId: objectType.id,
    title,
    status: objectType.status,
    properties: { [objectType.primaryField]: "" },
  };
}
