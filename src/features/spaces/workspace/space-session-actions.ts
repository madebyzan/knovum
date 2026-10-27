import { isEqual } from "es-toolkit";
import type { Ref } from "vue";

import { spaceMutationAdmissions, waitForSpaceMutations } from "../history/mutation-barrier";
import { releaseKnowledgeIndex, scheduleSpaceKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import type { Space } from "../model/space";
import { loadSpaceSessionProjection, type SpaceSessionContent } from "../session/space-projection";
import { emptySpaceContent, switchSpaceSession } from "./retained-space-sessions";
import { markSpaceOpenedAndTrack } from "./space-creation";
import type { SpaceSessionContext } from "./space-session-context";
import { referencedSpaceIds } from "./workbench-tab";
import { discardSpaceWorkbenchTabs } from "./workbench-tab-state";

export interface SpaceSessionSwitch {
  /** Applies the next Space session synchronously, so no render observes a mixed projection. */
  apply(): void;
  /** Whether the session shows content it retained, which work may have changed since. */
  retained: boolean;
}

export interface SpaceSessionLoad {
  complete(): Promise<SpaceSessionSwitch | undefined>;
}

/**
 * Claims the Space switch immediately, so repeated activations coalesce and the workbench can
 * show its loading state, then prepares the next session without activating it. A Space that is
 * open in another tab already holds its session, so only its opening is recorded; any other Space
 * loads its projection first. The caller applies the result together with its tab activation
 * inside a single transition.
 */
export function startSpaceSessionLoad(context: SpaceSessionContext, spaceId: string): SpaceSessionLoad {
  const generation = ++context.runtime.openGeneration;
  if (context.draftSpace.value?.id === spaceId) {
    context.runtime.pendingSpaceId = undefined;
    context.loading.value = false;
    const draft: SpaceSessionSwitch = {
      // A draft has no persisted projection, so its session starts from empty content.
      apply: () => {
        cancelPendingSpaceOpen(context);
        switchSpaceSession(context, spaceId, emptySpaceContent());
      },
      retained: false,
    };
    return { complete: async () => (generation === context.runtime.openGeneration ? draft : undefined) };
  }
  context.runtime.pendingSpaceId = spaceId;
  context.loading.value = true;
  return { complete: () => loadSpaceSession(context, spaceId, generation) };
}

async function loadSpaceSession(
  context: SpaceSessionContext,
  spaceId: string,
  generation: number,
): Promise<SpaceSessionSwitch | undefined> {
  try {
    const retained = context.runtime.retained.get(spaceId)?.content;
    const content = retained ?? (await loadSpaceSessionProjection(spaceId));
    if (generation !== context.runtime.openGeneration) return undefined;
    await markSpaceOpenedAndTrack(context, spaceId);
    if (generation !== context.runtime.openGeneration) return undefined;
    return {
      apply: () => {
        switchSpaceSession(context, spaceId, content);
        scheduleSpaceKnowledgeIndex(spaceId);
      },
      retained: Boolean(retained),
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
 * person using it. A read that a local edit overlapped may predate that edit, so it is read again
 * once the session is idle; only a read no edit overlapped replaces what the session shows.
 */
export async function refreshSpace(context: SpaceSessionContext) {
  const spaceId = context.activeSpaceId.value;
  if (!spaceId || context.runtime.pendingSpaceId) return;
  const generation = ++context.runtime.openGeneration;
  let admitted: number;
  let projection: SpaceSessionContent;
  do {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each read waits for the edits the previous one overlapped.
    await waitForSpaceMutations(context.history);
    admitted = spaceMutationAdmissions(context.history);
    // oxlint-disable-next-line eslint/no-await-in-loop -- Reads are sequential by design: only the last may apply.
    projection = await loadSpaceSessionProjection(spaceId);
    if (generation !== context.runtime.openGeneration || context.activeSpaceId.value !== spaceId) return;
  } while (admitted !== spaceMutationAdmissions(context.history));
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
    context.runtime.retained.delete(spaceId);
    releaseKnowledgeIndex(spaceId);
    if (context.draftSpace.value?.id !== spaceId) continue;
    context.draftSpace.value = undefined;
    discardSpaceWorkbenchTabs(context, spaceId);
  }
}
