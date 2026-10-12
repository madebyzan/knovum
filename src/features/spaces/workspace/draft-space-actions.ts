import { beginSpaceMutation } from "../history/mutation-barrier";
import { createSpaceDraft, persistAndTrackSpace } from "./space-creation";
import type { SpaceSessionContext } from "./space-session-context";

/** One session holds at most one unsaved draft Space; repeated new-Space actions reuse it. */
export function ensureDraftSpace(context: SpaceSessionContext, title: string) {
  const existing = context.draftSpace.value;
  if (existing) return existing;
  const space = createSpaceDraft(title, "space");
  context.draftSpace.value = space;
  return space;
}

export async function materializeActiveSpace(context: SpaceSessionContext) {
  const draft = context.draftSpace.value;
  if (!draft || draft.id !== context.activeSpaceId.value) return;
  const releaseMutation = beginSpaceMutation(context.history);
  try {
    await persistAndTrackSpace(context, draft);
    context.draftSpace.value = undefined;
  } finally {
    releaseMutation();
  }
}
