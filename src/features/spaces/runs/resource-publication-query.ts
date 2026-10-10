export function resourcePublicationQueryKey(spaceId: string) {
  return ["spaces", "resource-publications", spaceId] as const;
}
