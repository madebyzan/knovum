export const SPACE_KNOWLEDGE_QUERY_KEYS = {
  root: ["space-knowledge"] as const,
  space: (spaceId: string) => [...SPACE_KNOWLEDGE_QUERY_KEYS.root, "space", spaceId] as const,
  objectTypes: (spaceId: string) => [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "object-types"] as const,
  health: (spaceId: string) => [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "health"] as const,
  indexState: (spaceId: string) => [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "index-state"] as const,
  history: (spaceId: string, nodeId: string) =>
    [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "history", nodeId] as const,
  provenance: (spaceId: string, targetKind: "node" | "relation", targetId: string) =>
    [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "provenance", targetKind, targetId] as const,
  proposals: (spaceId: string) => [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "proposals"] as const,
  answerVerification: (messageId: string) =>
    [...SPACE_KNOWLEDGE_QUERY_KEYS.root, "answer-verification", messageId] as const,
  related: (spaceId: string, nodeId: string) =>
    [...SPACE_KNOWLEDGE_QUERY_KEYS.space(spaceId), "related", nodeId] as const,
};
