import { omit, pick } from "es-toolkit";

import type { Space } from "../model/space";

export function packageSpace(space: Space) {
  return pick(space, ["id", "kind", "title", "lastOpenedAt", "createdAt", "updatedAt"]);
}

export function restoreOwnedRow<Row extends { spaceId: string }>(
  row: Row,
  sourceSpaceId: string,
): Omit<Row, "spaceId"> {
  if (row.spaceId !== sourceSpaceId) throw new Error("Package row belongs to a different source space");
  return omit(row, ["spaceId"]);
}
