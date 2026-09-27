// Keep the cache identity stable when moving ownership out of Knowledge.
export const spaceDeletionQueryKeys = {
  deleted: (spaceId: string | null) => ["space-knowledge", "deleted", spaceId] as const,
};
