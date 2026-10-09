import { nextTick } from "vue";

import { beginSpaceMutation } from "../history/mutation-barrier";
import type { Space } from "../model/space";
import { recordWorkspaceNavigation } from "./navigation-history";
import { switchSpaceHistory } from "./session-history";
import { createSpaceDraft, persistAndTrackSpace } from "./space-creation";
import type { SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";

export async function createDraftSpace(context: SpaceSessionContext, title: string) {
  const existingDraft = context.draftSpace.value;
  if (existingDraft) return activateDraftSpace(context, existingDraft, true);
  const space = createSpaceDraft(title, "space");
  context.draftSpace.value = space;
  return activateDraftSpace(context, space, true);
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

export async function activateDraftSpace(
  context: SpaceSessionContext,
  space: Space,
  recordNavigation: boolean,
) {
  return transitionActiveSpaceSession(context, async () => {
    context.runtime.openGeneration += 1;
    context.runtime.pendingSpaceId = undefined;
    context.loading.value = false;
    if (context.focusedEditorId.value) {
      context.focusedEditorId.value = undefined;
      await nextTick();
    }
    switchSpaceHistory(context, space.id);
    context.activeSpaceId.value = space.id;
    context.nodes.value = [];
    context.editSnapshot.value = { editRevision: 0, structure: [], layouts: [], relationStyles: [] };
    context.groupScopePath.value = [];
    context.scenes.value = [];
    context.relations.value = [];
    if (!context.openSpaceIds.value.includes(space.id)) context.openSpaceIds.value.push(space.id);
    if (recordNavigation) recordWorkspaceNavigation(context, space.id);
    return true;
  });
}
