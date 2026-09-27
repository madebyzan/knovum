import type { Space } from "../model/space";
import { createSpace as persistSpace, markSpaceOpened } from "./space-operations";
import type { SpaceSessionContext } from "./space-session-context";

export function createSpaceDraft(title: string, kind: Space["kind"]): Space {
  const now = Date.now();
  return {
    id: crypto.randomUUID(),
    kind,
    title,
    archivedAt: null,
    favorite: false,
    thumbnailPath: null,
    nodeCount: 0,
    lastOpenedAt: now,
    createdAt: now,
    updatedAt: now,
  };
}

export async function createAndTrackSpace(context: SpaceSessionContext, title: string, kind: Space["kind"]) {
  const space = createSpaceDraft(title, kind);
  await persistSpace(space);
  context.spaces.value.push(space);
  return space;
}

export async function persistAndTrackSpace(context: SpaceSessionContext, space: Space) {
  await persistSpace(space);
  context.spaces.value.push(space);
}

export async function markSpaceOpenedAndTrack(context: SpaceSessionContext, id: string) {
  const openedAt = Date.now();
  const space = context.spaces.value.find((candidate) => candidate.id === id);
  if (space) space.lastOpenedAt = openedAt;
  await markSpaceOpened(id);
}
