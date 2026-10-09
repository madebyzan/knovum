import { difference, without } from "es-toolkit";
import { nextTick } from "vue";

import { releaseKnowledgeIndex, scheduleSpaceKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { loadSpaceSessionProjection } from "../session/space-projection";
import { activateDraftSpace, createDraftSpace } from "./draft-space-actions";
import { recordWorkspaceNavigation, removeWorkspaceNavigation } from "./navigation-history";
import { switchSpaceHistory } from "./session-history";
import { createAndTrackSpace, markSpaceOpenedAndTrack } from "./space-creation";
import { listSpaces } from "./space-operations";
import type { LoadSpaceOptions, SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";

export async function initializeSpaces(context: SpaceSessionContext, inboxTitle: string) {
  if (context.ready.value) return;
  context.spaces.value = await listSpaces();
  let inbox = context.spaces.value.find((space) => space.kind === "inbox");
  if (!inbox) inbox = await createAndTrackSpace(context, inboxTitle, "inbox");
  await openSpace(context, inbox.id);
  context.ready.value = true;
}

export async function createSpace(context: SpaceSessionContext, title: string) {
  await createDraftSpace(context, title);
}

export async function openSpace(context: SpaceSessionContext, spaceId: string) {
  await activateSpace(context, spaceId, true);
}

async function activateSpace(context: SpaceSessionContext, spaceId: string, recordNavigation: boolean) {
  if (context.runtime.pendingSpaceId === spaceId) return false;
  if (context.activeSpaceId.value === spaceId) {
    if (context.runtime.pendingSpaceId) cancelPendingSpaceOpen(context);
    return true;
  }
  if (context.draftSpace.value?.id === spaceId)
    return activateDraftSpace(context, context.draftSpace.value, recordNavigation);
  const space = context.spaces.value.find(
    (candidate) => candidate.id === spaceId && candidate.archivedAt === null,
  );
  if (!space) throw new Error(`Cannot open unavailable space: ${spaceId}`);
  const activated = await loadSpace(context, spaceId, { markOpened: true, resetHistory: true });
  if (!activated) return false;
  if (!context.openSpaceIds.value.includes(spaceId)) context.openSpaceIds.value.push(spaceId);
  if (recordNavigation) recordWorkspaceNavigation(context, spaceId);
  return true;
}

export async function closeSpaceTab(context: SpaceSessionContext, spaceId: string) {
  const index = context.openSpaceIds.value.indexOf(spaceId);
  if (index < 0 || context.openSpaceIds.value.length === 1) return;
  if (context.activeSpaceId.value === spaceId) {
    const nextId = context.openSpaceIds.value[index + 1] ?? context.openSpaceIds.value[index - 1];
    if (nextId && !(await activateSpace(context, nextId, false))) return;
  }
  context.openSpaceIds.value.splice(index, 1);
  if (context.draftSpace.value?.id === spaceId) context.draftSpace.value = undefined;
  else context.closedSpaceIds.value = [spaceId, ...without(context.closedSpaceIds.value, spaceId)];
  leaveSpaceSession(context, [spaceId]);
}

export async function closeOtherSpaceTabs(context: SpaceSessionContext, spaceId: string) {
  if (!context.openSpaceIds.value.includes(spaceId)) return;
  const closed = without(context.openSpaceIds.value, spaceId);
  if (!(await activateSpace(context, spaceId, false))) return;
  context.openSpaceIds.value = [spaceId];
  retireClosedSpaceTabs(context, closed);
}

export async function closeSpaceTabsToRight(context: SpaceSessionContext, spaceId: string) {
  const index = context.openSpaceIds.value.indexOf(spaceId);
  if (index < 0) return;
  const closed = context.openSpaceIds.value.slice(index + 1);
  if (closed.includes(context.activeSpaceId.value ?? "") && !(await activateSpace(context, spaceId, false)))
    return;
  context.openSpaceIds.value = context.openSpaceIds.value.slice(0, index + 1);
  retireClosedSpaceTabs(context, closed);
}

export async function reopenClosedSpaceTab(context: SpaceSessionContext) {
  const id = context.closedSpaceIds.value[0];
  if (!id) return;
  if (!(await activateSpace(context, id, true))) return;
  context.closedSpaceIds.value.shift();
}

export function discardSpaceTab(context: SpaceSessionContext, spaceId: string) {
  context.openSpaceIds.value = without(context.openSpaceIds.value, spaceId);
  context.closedSpaceIds.value = without(context.closedSpaceIds.value, spaceId);
  if (context.draftSpace.value?.id === spaceId) context.draftSpace.value = undefined;
  leaveSpaceSession(context, [spaceId]);
}

function retireClosedSpaceTabs(context: SpaceSessionContext, closed: readonly string[]) {
  const draftId = context.draftSpace.value?.id;
  if (draftId && closed.includes(draftId)) context.draftSpace.value = undefined;
  context.closedSpaceIds.value = [
    ...(draftId ? without(closed, draftId) : closed).toReversed(),
    ...difference(context.closedSpaceIds.value, closed),
  ];
  leaveSpaceSession(context, closed);
}

function leaveSpaceSession(context: SpaceSessionContext, spaceIds: readonly string[]) {
  for (const id of spaceIds) {
    context.runtime.historyBySpace.delete(id);
    removeWorkspaceNavigation(context, id);
    releaseKnowledgeIndex(id);
  }
}

export async function navigateSpaceHistory(context: SpaceSessionContext, direction: -1 | 1) {
  const nextIndex = context.navigationIndex.value + direction;
  const id = context.navigationHistory.value[nextIndex];
  if (!id) return;
  if (!(await activateSpace(context, id, false))) return;
  context.navigationIndex.value = nextIndex;
}

export async function refreshSpace(context: SpaceSessionContext) {
  const id = context.activeSpaceId.value;
  if (!id || context.runtime.pendingSpaceId) return;
  await loadSpace(context, id, { markOpened: false, resetHistory: false });
}

async function loadSpace(context: SpaceSessionContext, spaceId: string, options: LoadSpaceOptions) {
  const generation = ++context.runtime.openGeneration;
  context.runtime.pendingSpaceId = spaceId;
  context.loading.value = true;
  try {
    return await transitionActiveSpaceSession(context, async () => {
      const { nodes, scenes, relations, editSnapshot } = await loadSpaceSessionProjection(spaceId);
      if (generation !== context.runtime.openGeneration) return false;
      if (options.markOpened) await markSpaceOpenedAndTrack(context, spaceId);
      if (generation !== context.runtime.openGeneration) return false;
      if (options.resetHistory) switchSpaceHistory(context, spaceId);
      if (context.focusedEditorId.value) {
        context.focusedEditorId.value = undefined;
        await nextTick();
      }
      context.activeSpaceId.value = spaceId;
      context.nodes.value = nodes;
      context.scenes.value = scenes;
      context.relations.value = relations;
      context.editSnapshot.value = editSnapshot;
      context.groupScopePath.value = [];
      scheduleSpaceKnowledgeIndex(spaceId);
      return true;
    });
  } finally {
    if (generation === context.runtime.openGeneration) {
      context.runtime.pendingSpaceId = undefined;
      context.loading.value = false;
    }
  }
}

function cancelPendingSpaceOpen(context: SpaceSessionContext) {
  context.runtime.openGeneration += 1;
  context.runtime.pendingSpaceId = undefined;
  context.loading.value = false;
}
