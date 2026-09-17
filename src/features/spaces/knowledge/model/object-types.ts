export const KNOWLEDGE_OBJECT_STATUSES = [
  "draft",
  "accepted",
  "contested",
  "superseded",
  "invalidated",
  "open",
  "resolved",
] as const;

export type SpaceKnowledgeObjectStatus = (typeof KNOWLEDGE_OBJECT_STATUSES)[number];

export type SpaceObjectFieldKind = "text" | "number" | "boolean" | "date" | "node-ref" | "enum";

export interface SpaceObjectFieldDefinition {
  key: string;
  label: string;
  kind: SpaceObjectFieldKind;
  required?: boolean;
  options?: string[];
}

export interface SpaceKnowledgeObjectType {
  id: string;
  spaceId: string | null;
  name: string;
  icon: string;
  description: string;
  fields: SpaceObjectFieldDefinition[];
  system: boolean;
  createdAt: number;
  updatedAt: number;
}

export const SYSTEM_OBJECT_TYPE_IDS = {
  claim: "system:claim",
  evidence: "system:evidence",
  question: "system:question",
  decision: "system:decision",
  concept: "system:concept",
  experiment: "system:experiment",
} as const;

export type SystemObjectTypeId = (typeof SYSTEM_OBJECT_TYPE_IDS)[keyof typeof SYSTEM_OBJECT_TYPE_IDS];

export type SystemKnowledgeObjectKind = keyof typeof SYSTEM_OBJECT_TYPE_IDS;

export const SYSTEM_OBJECT_TYPES = {
  claim: { id: SYSTEM_OBJECT_TYPE_IDS.claim, status: "draft", primaryField: "statement" },
  evidence: { id: SYSTEM_OBJECT_TYPE_IDS.evidence, status: "draft", primaryField: "observation" },
  question: { id: SYSTEM_OBJECT_TYPE_IDS.question, status: "open", primaryField: "question" },
  decision: { id: SYSTEM_OBJECT_TYPE_IDS.decision, status: "draft", primaryField: "decision" },
  concept: { id: SYSTEM_OBJECT_TYPE_IDS.concept, status: "draft", primaryField: "definition" },
  experiment: { id: SYSTEM_OBJECT_TYPE_IDS.experiment, status: "draft", primaryField: "hypothesis" },
} as const satisfies Record<
  SystemKnowledgeObjectKind,
  { id: SystemObjectTypeId; status: SpaceKnowledgeObjectStatus; primaryField: string }
>;

export function systemKnowledgeObjectKind(objectTypeId: string): SystemKnowledgeObjectKind | undefined {
  return (Object.entries(SYSTEM_OBJECT_TYPES) as [SystemKnowledgeObjectKind, { id: string }][]).find(
    ([, definition]) => definition.id === objectTypeId,
  )?.[0];
}

export function knowledgeObjectStatusesForType(objectTypeId: string): readonly SpaceKnowledgeObjectStatus[] {
  if (objectTypeId === SYSTEM_OBJECT_TYPE_IDS.question) return ["open", "resolved", "invalidated"];
  if (objectTypeId === SYSTEM_OBJECT_TYPE_IDS.decision)
    return ["draft", "accepted", "superseded", "invalidated"];
  if (objectTypeId === SYSTEM_OBJECT_TYPE_IDS.experiment) return ["draft", "accepted", "invalidated"];
  if (
    objectTypeId === SYSTEM_OBJECT_TYPE_IDS.claim ||
    objectTypeId === SYSTEM_OBJECT_TYPE_IDS.evidence ||
    objectTypeId === SYSTEM_OBJECT_TYPE_IDS.concept
  ) {
    return ["draft", "accepted", "contested", "superseded", "invalidated"];
  }
  return KNOWLEDGE_OBJECT_STATUSES;
}
