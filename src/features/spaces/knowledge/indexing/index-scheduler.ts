import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import { getLocalResourcesStatus, textEmbeddingReady } from "../../../../foundation/runtime/local-resources";
import { indexKnowledgeRetrieval } from "../persistence/retrieval-index";
import { isKnowledgeIndexingBusy } from "./indexing-outcome";

export type KnowledgeIndexStatus = "idle" | "scheduled" | "indexing" | "ready" | "unprepared" | "unavailable";
type StatusListener = (status: KnowledgeIndexStatus) => void;

interface SpaceIndexState {
  generation: number;
  nodeIds: Set<string>;
  full: boolean;
  running: boolean;
  consecutiveFailures: number;
  timer: ReturnType<typeof setTimeout> | undefined;
}

const stateBySpaceId = new Map<string, SpaceIndexState>();
const statusBySpaceId = new Map<string, KnowledgeIndexStatus>();
const listenersBySpaceId = new Map<string, Set<StatusListener>>();
const DEBOUNCE_MS = 5_000;
const MAX_RETRY_DELAY_MS = 60_000;
const MAX_ATTEMPT_RETRIES = 3;

export function scheduleKnowledgeIndex(spaceId: string, nodeIds: readonly string[]) {
  if (!nodeIds.length) return;
  const state = stateFor(spaceId);
  state.generation += 1;
  if (!state.full) for (const id of nodeIds) state.nodeIds.add(id);
  schedulePending(spaceId, state);
}

export function scheduleSpaceKnowledgeIndex(spaceId: string) {
  const state = stateFor(spaceId);
  state.generation += 1;
  state.full = true;
  state.nodeIds.clear();
  schedulePending(spaceId, state);
}

export function releaseKnowledgeIndex(spaceId: string) {
  const state = stateBySpaceId.get(spaceId);
  if (state?.timer) clearTimeout(state.timer);
  stateBySpaceId.delete(spaceId);
  statusBySpaceId.delete(spaceId);
}

export function knowledgeIndexStatus(spaceId: string) {
  return statusBySpaceId.get(spaceId) ?? "idle";
}

export function subscribeKnowledgeIndexStatus(spaceId: string, listener: StatusListener) {
  const spaceListeners = listenersBySpaceId.get(spaceId) ?? new Set<StatusListener>();
  spaceListeners.add(listener);
  listenersBySpaceId.set(spaceId, spaceListeners);
  listener(knowledgeIndexStatus(spaceId));
  return () => {
    spaceListeners.delete(listener);
    if (!spaceListeners.size) listenersBySpaceId.delete(spaceId);
  };
}

function stateFor(spaceId: string) {
  const existing = stateBySpaceId.get(spaceId);
  if (existing) return existing;
  const state: SpaceIndexState = {
    generation: 0,
    nodeIds: new Set(),
    full: false,
    running: false,
    consecutiveFailures: 0,
    timer: undefined,
  };
  stateBySpaceId.set(spaceId, state);
  return state;
}

function schedulePending(
  spaceId: string,
  state: SpaceIndexState,
  delayMs = DEBOUNCE_MS,
  status: KnowledgeIndexStatus = "scheduled",
) {
  if (state.running) {
    setStatus(spaceId, status);
    return;
  }
  if (state.timer) clearTimeout(state.timer);
  state.timer = setTimeout(() => void runPending(spaceId, state), delayMs);
  setStatus(spaceId, status);
}

interface ClaimedScope {
  generation: number;
  full: boolean;
  nodeIds: string[] | undefined;
}

function claimPendingScope(state: SpaceIndexState): ClaimedScope {
  const full = state.full;
  const claimed = { generation: state.generation, full, nodeIds: full ? undefined : [...state.nodeIds] };
  state.full = false;
  state.nodeIds.clear();
  return claimed;
}

// The scope an attempt claimed is still unindexed when it fails, so it returns to the pending set.
// Losing it would leave those nodes out of the index until something happened to touch them again.
function returnPendingScope(state: SpaceIndexState, claimed: ClaimedScope) {
  state.full ||= claimed.full;
  for (const id of claimed.nodeIds ?? []) state.nodeIds.add(id);
}

async function runPending(spaceId: string, state: SpaceIndexState) {
  if (state.running) return;
  state.timer = undefined;
  const claimed = claimPendingScope(state);
  state.running = true;
  let outcome: KnowledgeIndexStatus = "unavailable";
  let failed = false;
  let deferred = false;
  try {
    outcome = await indexIfReady(spaceId, state, claimed.nodeIds);
  } catch (error) {
    // A pass somebody started holds this Space. It is doing the same work, so this attempt waits
    // for its turn instead of counting a failure it did not have.
    deferred = isKnowledgeIndexingBusy(error);
    failed = !deferred;
    returnPendingScope(state, claimed);
    if (!deferred) void logError(`Knowledge semantic indexing failed: ${formatErrorDiagnostic(error)}`);
  } finally {
    state.running = false;
  }
  state.consecutiveFailures = failed ? state.consecutiveFailures + 1 : 0;
  // The space can be released while its run is in flight; a released state must not reschedule or
  // write a status, or it would resurrect the entry and clobber a later session for the same space.
  if (stateBySpaceId.get(spaceId) !== state) return;
  if (deferred) {
    schedulePending(spaceId, state, DEBOUNCE_MS, "indexing");
    return;
  }
  settleAttempt(spaceId, state, {
    outcome,
    failed,
    hasNewerWork: state.generation !== claimed.generation,
  });
}

interface AttemptSettlement {
  outcome: KnowledgeIndexStatus;
  failed: boolean;
  hasNewerWork: boolean;
}

function settleAttempt(
  spaceId: string,
  state: SpaceIndexState,
  { outcome, failed, hasNewerWork }: AttemptSettlement,
) {
  if (failed) {
    // A pending retry does not make the index available, so the failure stays visible until an
    // attempt succeeds. Retries are limited because a permanently unusable embedding runtime must
    // not be asked again forever; later edits are new evidence and are always attempted.
    if (state.consecutiveFailures <= MAX_ATTEMPT_RETRIES || hasNewerWork) {
      schedulePending(spaceId, state, retryDelayMs(state.consecutiveFailures), outcome);
      return;
    }
    setStatus(spaceId, outcome);
    return;
  }
  if (hasNewerWork) {
    schedulePending(spaceId, state, 0);
    return;
  }
  setStatus(spaceId, outcome);
}

// Indexing shares one local inference worker with transcription and synthesis. Retrying a failed
// attempt immediately spent that worker on work that was about to fail again, which is how a
// single failure became an unbroken stream of them.
function retryDelayMs(consecutiveFailures: number) {
  return Math.min(DEBOUNCE_MS * 2 ** Math.max(consecutiveFailures - 1, 0), MAX_RETRY_DELAY_MS);
}

async function indexIfReady(
  spaceId: string,
  state: SpaceIndexState,
  nodeIds: string[] | undefined,
): Promise<KnowledgeIndexStatus> {
  const resources = await getLocalResourcesStatus();
  if (stateBySpaceId.get(spaceId) !== state || !textEmbeddingReady(resources)) return "unprepared";
  setStatus(spaceId, "indexing");
  // Upkeep after an edit is not a task somebody started: it reports through this Space's index
  // status, which the Space already shows, instead of filling the Task Center with its own debounce.
  await indexKnowledgeRetrieval({
    spaceId,
    nodeIds,
    onProgress: () => {},
  });
  return "ready";
}

function setStatus(spaceId: string, status: KnowledgeIndexStatus) {
  statusBySpaceId.set(spaceId, status);
  for (const listener of listenersBySpaceId.get(spaceId) ?? []) listener(status);
}
