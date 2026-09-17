export const spaceKnowledgeQueryKeys = {
  root: ["space-knowledge"] as const,
  space: (spaceId: string) => [...spaceKnowledgeQueryKeys.root, "space", spaceId] as const,
  objectTypes: (spaceId: string) => [...spaceKnowledgeQueryKeys.space(spaceId), "object-types"] as const,
  health: (spaceId: string) => [...spaceKnowledgeQueryKeys.space(spaceId), "health"] as const,
  history: (spaceId: string, nodeId: string) =>
    [...spaceKnowledgeQueryKeys.space(spaceId), "history", nodeId] as const,
  provenance: (spaceId: string, targetKind: "node" | "relation", targetId: string) =>
    [...spaceKnowledgeQueryKeys.space(spaceId), "provenance", targetKind, targetId] as const,
  deleted: (spaceId: string | null) => [...spaceKnowledgeQueryKeys.root, "deleted", spaceId] as const,
};
