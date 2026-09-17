import { error as logError } from "@tauri-apps/plugin-log";

import { decodeAiErrorPayload } from "../../../../foundation/ai/ai-runtime";
import { indexKnowledgeRetrieval } from "../persistence/embedding-repository";

export type KnowledgeIndexStatus = "idle" | "scheduled" | "indexing" | "ready" | "unavailable";
type StatusListener = (status: KnowledgeIndexStatus) => void;

interface SpaceIndexState {
  generation: number;
  nodeIds: Set<string>;
  full: boolean;
  running: boolean;
  timer: ReturnType<typeof setTimeout> | undefined;
}

const states = new Map<string, SpaceIndexState>();
const statuses = new Map<string, KnowledgeIndexStatus>();
const listeners = new Map<string, Set<StatusListener>>();
const DEBOUNCE_MS = 5_000;

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
  const state = states.get(spaceId);
  if (state?.timer) clearTimeout(state.timer);
  states.delete(spaceId);
  statuses.delete(spaceId);
}

export function knowledgeIndexStatus(spaceId: string) {
  return statuses.get(spaceId) ?? "idle";
}

export function subscribeKnowledgeIndexStatus(spaceId: string, listener: StatusListener) {
  const spaceListeners = listeners.get(spaceId) ?? new Set<StatusListener>();
  spaceListeners.add(listener);
  listeners.set(spaceId, spaceListeners);
  listener(knowledgeIndexStatus(spaceId));
  return () => {
    spaceListeners.delete(listener);
    if (!spaceListeners.size) listeners.delete(spaceId);
  };
}

function stateFor(spaceId: string) {
  const existing = states.get(spaceId);
  if (existing) return existing;
  const state: SpaceIndexState = {
    generation: 0,
    nodeIds: new Set(),
    full: false,
    running: false,
    timer: undefined,
  };
  states.set(spaceId, state);
  return state;
}

function schedulePending(spaceId: string, state: SpaceIndexState, delay = DEBOUNCE_MS) {
  if (state.running) {
    setStatus(spaceId, "scheduled");
    return;
  }
  if (state.timer) clearTimeout(state.timer);
  state.timer = setTimeout(() => void runPending(spaceId, state), delay);
  setStatus(spaceId, "scheduled");
}

async function runPending(spaceId: string, state: SpaceIndexState) {
  if (state.running) return;
  state.timer = undefined;
  const generation = state.generation;
  const full = state.full;
  const nodeIds = full ? undefined : [...state.nodeIds];
  state.full = false;
  state.nodeIds.clear();
  state.running = true;
  setStatus(spaceId, "indexing");
  let failed = false;
  try {
    await indexKnowledgeRetrieval(spaceId, nodeIds);
  } catch (error) {
    failed = true;
    void logError(`Knowledge semantic indexing failed: ${describeIndexError(error)}`);
  } finally {
    state.running = false;
  }

  const hasNewerWork = state.generation !== generation || state.full || state.nodeIds.size > 0;
  // The space can be released while its run is in flight; a released state must not reschedule or
  // write a status, or it would resurrect the entry and clobber a later session for the same space.
  if (states.get(spaceId) !== state) return;
  if (hasNewerWork) {
    schedulePending(spaceId, state, 0);
    return;
  }
  setStatus(spaceId, failed ? "unavailable" : "ready");
}

function describeIndexError(error: unknown) {
  const aiError = decodeAiErrorPayload(error);
  if (aiError) {
    return JSON.stringify({ code: aiError.code, message: aiError.message });
  }
  if (error instanceof Error) return error.message;
  return String(error);
}

function setStatus(spaceId: string, status: KnowledgeIndexStatus) {
  statuses.set(spaceId, status);
  for (const listener of listeners.get(spaceId) ?? []) listener(status);
}
