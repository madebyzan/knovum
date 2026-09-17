import { nextTick } from "vue";

import { clearSpaceHistory } from "../history/stack";
import { releaseKnowledgeIndex, scheduleSpaceKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { listRelations } from "../knowledge/relations/relation-repository";
import { listNodes } from "../nodes/node-repository";
import { listScenes } from "../scenes/scene-repository";
import { activateDraftSpace, createDraftSpace } from "./draft-space-actions";
import { recordWorkspaceNavigation, removeWorkspaceNavigation } from "./navigation-history";
import { createAndTrackSpace, markSpaceOpenedAndTrack } from "./space-creation";
import { listSpaces } from "./space-repository";
import type { LoadSpaceOptions, SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";

export type { SpaceSessionContext } from "./space-session-context";

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

export { materializeActiveSpace } from "./draft-space-actions";
export { renameSpace } from "./space-title-actions";

export async function openSpace(context: SpaceSessionContext, id: string) {
  await activateSpace(context, id, true);
}

async function activateSpace(context: SpaceSessionContext, id: string, recordNavigation: boolean) {
  if (context.runtime.pendingSpaceId === id) return false;
  if (context.activeSpaceId.value === id) {
    if (context.runtime.pendingSpaceId) cancelPendingSpaceOpen(context);
    return true;
  }
  if (context.draftSpace.value?.id === id)
    return activateDraftSpace(context, context.draftSpace.value, recordNavigation);
  const space = context.spaces.value.find(
    (candidate) => candidate.id === id && candidate.archivedAt === null,
  );
  if (!space) throw new Error(`Cannot open unavailable space: ${id}`);
  const activated = await loadSpace(context, id, { markOpened: true, resetHistory: true });
  if (!activated) return false;
  if (!context.openSpaceIds.value.includes(id)) context.openSpaceIds.value.push(id);
  if (recordNavigation) recordWorkspaceNavigation(context, id);
  return true;
}

export async function closeSpaceTab(context: SpaceSessionContext, id: string) {
  const index = context.openSpaceIds.value.indexOf(id);
  if (index < 0 || context.openSpaceIds.value.length === 1) return;
  if (context.activeSpaceId.value === id) {
    const nextId = context.openSpaceIds.value[index + 1] ?? context.openSpaceIds.value[index - 1];
    if (nextId && !(await activateSpace(context, nextId, false))) return;
  }
  context.openSpaceIds.value.splice(index, 1);
  if (context.draftSpace.value?.id === id) context.draftSpace.value = undefined;
  else
    context.closedSpaceIds.value = [
      id,
      ...context.closedSpaceIds.value.filter((candidate) => candidate !== id),
    ];
  leaveSpaceSession(context, [id]);
}

export async function closeOtherSpaceTabs(context: SpaceSessionContext, id: string) {
  if (!context.openSpaceIds.value.includes(id)) return;
  const closed = context.openSpaceIds.value.filter((candidate) => candidate !== id);
  if (!(await activateSpace(context, id, false))) return;
  context.openSpaceIds.value = [id];
  retireClosedSpaceTabs(context, closed);
}

export async function closeSpaceTabsToRight(context: SpaceSessionContext, id: string) {
  const index = context.openSpaceIds.value.indexOf(id);
  if (index < 0) return;
  const closed = context.openSpaceIds.value.slice(index + 1);
  if (closed.includes(context.activeSpaceId.value ?? "") && !(await activateSpace(context, id, false)))
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

export function discardSpaceTab(context: SpaceSessionContext, id: string) {
  context.openSpaceIds.value = context.openSpaceIds.value.filter((candidate) => candidate !== id);
  context.closedSpaceIds.value = context.closedSpaceIds.value.filter((candidate) => candidate !== id);
  if (context.draftSpace.value?.id === id) context.draftSpace.value = undefined;
  leaveSpaceSession(context, [id]);
}

function retireClosedSpaceTabs(context: SpaceSessionContext, closed: readonly string[]) {
  const draftId = context.draftSpace.value?.id;
  if (draftId && closed.includes(draftId)) context.draftSpace.value = undefined;
  context.closedSpaceIds.value = [
    ...closed.filter((candidate) => candidate !== draftId).toReversed(),
    ...context.closedSpaceIds.value.filter((candidate) => !closed.includes(candidate)),
  ];
  leaveSpaceSession(context, closed);
}

function leaveSpaceSession(context: SpaceSessionContext, ids: readonly string[]) {
  for (const id of ids) {
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

async function loadSpace(context: SpaceSessionContext, id: string, options: LoadSpaceOptions) {
  return transitionActiveSpaceSession(context, async () => {
    const generation = ++context.runtime.openGeneration;
    context.runtime.pendingSpaceId = id;
    context.loading.value = true;
    try {
      const [nodes, scenes, relations] = await Promise.all([
        listNodes(id),
        listScenes(id),
        listRelations(id),
      ]);
      if (generation !== context.runtime.openGeneration) return false;
      if (options.resetHistory) clearSpaceHistory(context.history);
      if (context.focusedEditorId.value) {
        context.focusedEditorId.value = undefined;
        await nextTick();
      }
      context.activeSpaceId.value = id;
      context.nodes.value = nodes;
      context.scenes.value = scenes;
      context.relations.value = relations;
      if (options.markOpened) await markSpaceOpenedAndTrack(context, id);
      scheduleSpaceKnowledgeIndex(id);
      return true;
    } finally {
      if (generation === context.runtime.openGeneration) {
        context.runtime.pendingSpaceId = undefined;
        context.loading.value = false;
      }
    }
  });
}

function cancelPendingSpaceOpen(context: SpaceSessionContext) {
  context.runtime.openGeneration += 1;
  context.runtime.pendingSpaceId = undefined;
  context.loading.value = false;
}
