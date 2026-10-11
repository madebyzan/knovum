// Keep the cache identity stable when moving ownership out of Knowledge.
export const SPACE_DELETION_QUERY_KEYS = {
  deleted: (spaceId: string | null) => ["space-knowledge", "deleted", spaceId] as const,
};
