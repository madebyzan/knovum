import { omit, pick } from "es-toolkit";

import type { Space } from "../model/space";

export function prepareSpaceRestore(space: Space) {
  return pick(space, ["id", "kind", "title", "lastOpenedAt", "createdAt", "updatedAt"]);
}

export function stripSourceSpace<SpaceOwnedContent extends { spaceId: string }>(
  content: SpaceOwnedContent,
  sourceSpaceId: string,
): Omit<SpaceOwnedContent, "spaceId"> {
  if (content.spaceId !== sourceSpaceId) throw new Error("Package row belongs to a different source space");
  return omit(content, ["spaceId"]);
}
