import { isEqual } from "es-toolkit";
import type { Ref } from "vue";

import { waitForSpaceMutations } from "../history/mutation-barrier";
import { releaseKnowledgeIndex, scheduleSpaceKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import type { Space } from "../model/space";
import { loadSpaceSessionProjection } from "../session/space-projection";
import { switchSpaceHistory } from "./session-history";
import { markSpaceOpenedAndTrack } from "./space-creation";
import type { LoadSpaceOptions, SpaceSessionContext } from "./space-session-context";
import { referencedSpaceIds } from "./workbench-tab";
import { discardSpaceWorkbenchTabs } from "./workbench-tab-state";

/** Applies a loaded Space session synchronously, so no render observes a mixed projection. */
type ApplySpaceSession = () => void;

export interface SpaceSessionLoad {
  complete(): Promise<ApplySpaceSession | undefined>;
}

/**
 * Claims the Space switch immediately, so repeated activations coalesce and the workbench can
 * show its loading state, then loads the projection without activating it. The caller applies the
 * result together with its tab activation inside a single transition.
 */
export function startSpaceSessionLoad(
  context: SpaceSessionContext,
  spaceId: string,
  options: LoadSpaceOptions,
): SpaceSessionLoad {
  const generation = ++context.runtime.openGeneration;
  if (context.draftSpace.value?.id === spaceId) {
    context.runtime.pendingSpaceId = undefined;
    context.loading.value = false;
    const applyDraft = prepareDraftSpaceSession(context);
    return { complete: async () => (generation === context.runtime.openGeneration ? applyDraft : undefined) };
  }
  context.runtime.pendingSpaceId = spaceId;
  context.loading.value = true;
  return { complete: () => loadSpaceSession(context, spaceId, generation, options) };
}

async function loadSpaceSession(
  context: SpaceSessionContext,
  spaceId: string,
  generation: number,
  options: LoadSpaceOptions,
): Promise<ApplySpaceSession | undefined> {
  try {
    const projection = await loadSpaceSessionProjection(spaceId);
    if (generation !== context.runtime.openGeneration) return undefined;
    if (options.markOpened) await markSpaceOpenedAndTrack(context, spaceId);
    if (generation !== context.runtime.openGeneration) return undefined;
    return () => {
      if (options.resetHistory) switchSpaceHistory(context, spaceId);
      context.nodes.value = projection.nodes;
      context.scenes.value = projection.scenes;
      context.relations.value = projection.relations;
      context.editSnapshot.value = projection.editSnapshot;
      context.groupScopePath.value = [];
      scheduleSpaceKnowledgeIndex(spaceId);
    };
  } finally {
    if (generation === context.runtime.openGeneration) {
      context.runtime.pendingSpaceId = undefined;
      context.loading.value = false;
    }
  }
}

export function cancelPendingSpaceOpen(context: SpaceSessionContext) {
  context.runtime.openGeneration += 1;
  context.runtime.pendingSpaceId = undefined;
  context.loading.value = false;
}

/** A draft Space has no persisted projection, so its session starts from empty content. */
function prepareDraftSpaceSession(context: SpaceSessionContext): ApplySpaceSession {
  return () => {
    cancelPendingSpaceOpen(context);
    const draftId = context.draftSpace.value?.id;
    if (draftId) switchSpaceHistory(context, draftId);
    context.nodes.value = [];
    context.scenes.value = [];
    context.relations.value = [];
    context.editSnapshot.value = { editRevision: 0, structure: [], layouts: [], relationStyles: [] };
    context.groupScopePath.value = [];
  };
}

export function requireAvailableSpace(context: SpaceSessionContext, spaceId: string): Space {
  const draft = context.draftSpace.value;
  if (draft?.id === spaceId) return draft;
  const space = context.spaces.value.find(
    (candidate) => candidate.id === spaceId && candidate.archivedAt === null,
  );
  if (!space) throw new Error(`Cannot open unavailable space: ${spaceId}`);
  return space;
}

/**
 * Reloads the projection of the Space that is already open.
 *
 * A reload is not a navigation: the canvas session stays mounted, edits stay admitted, and the
 * view changes only where the content changed. Work that publishes into an open Space can
 * therefore report itself as often as it likes without the session flickering underneath the
 * person using it.
 */
export async function refreshSpace(context: SpaceSessionContext) {
  const spaceId = context.activeSpaceId.value;
  if (!spaceId || context.runtime.pendingSpaceId) return;
  const generation = ++context.runtime.openGeneration;
  await waitForSpaceMutations(context.history);
  const projection = await loadSpaceSessionProjection(spaceId);
  if (
    generation !== context.runtime.openGeneration ||
    context.activeSpaceId.value !== spaceId ||
    // A slower read can return an older Space than the session already holds.
    projection.editSnapshot.editRevision < context.editSnapshot.value.editRevision
  )
    return;
  applyChanged(context.nodes, projection.nodes);
  applyChanged(context.scenes, projection.scenes);
  applyChanged(context.relations, projection.relations);
  applyChanged(context.editSnapshot, projection.editSnapshot);
}

/** Keeps what the reload found identical, so unchanged content keeps its rendered identity. */
function applyChanged<T>(target: Ref<T>, next: T) {
  if (!isEqual(target.value, next)) target.value = next;
}

/** Releases the retained session of every Space the remaining tabs no longer reference. */
export function releaseUnreferencedSpaces(context: SpaceSessionContext, previous: readonly string[]) {
  const referenced = new Set(referencedSpaceIds(context.tabs.value));
  for (const spaceId of previous) {
    if (referenced.has(spaceId)) continue;
    context.runtime.historyBySpace.delete(spaceId);
    releaseKnowledgeIndex(spaceId);
    if (context.draftSpace.value?.id !== spaceId) continue;
    context.draftSpace.value = undefined;
    discardSpaceWorkbenchTabs(context, spaceId);
  }
}
