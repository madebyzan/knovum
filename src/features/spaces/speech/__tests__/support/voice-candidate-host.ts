import { vi } from "vitest";
import type * as VueI18n from "vue-i18n";

import type { RunObservation } from "../../../contracts/generated/RunObservation";
import type * as observation from "../../../runs/resource-run-observation";
import type * as speechIpc from "../../ipc";
import type * as candidateIpc from "../../voice-candidate-ipc";

/**
 * The host a voice candidate talks to: candidate IPC, speech cancellation and run observation.
 * Suites route their `vi.mock` factories for those modules here.
 */
export const voiceHost = {
  draftVoiceCandidate: vi.fn<typeof candidateIpc.draftVoiceCandidate>(),
  getActiveVoiceCandidateRun: vi.fn<typeof candidateIpc.getActiveVoiceCandidateRun>(),
  listVoiceCandidates: vi.fn<typeof candidateIpc.listVoiceCandidates>(),
  getVoiceCandidate: vi.fn<typeof candidateIpc.getVoiceCandidate>(),
  cancelSpeechGeneration: vi.fn<typeof speechIpc.cancelSpeechGeneration>(),
  observeResourceRun: vi.fn<typeof observation.observeResourceRun>(),
  cancelResourceRun: vi.fn<typeof observation.cancelResourceRun>(),
};

/** vue-i18n whose translations are their keys. */
export async function keyedI18n(original: <T>() => Promise<T>) {
  return { ...(await original<typeof VueI18n>()), useI18n: () => ({ t: (key: string) => key }) };
}

/** A candidate run the backend reports as still running, identified by `id`. */
export function runningRun(id: string): RunObservation {
  return {
    id,
    scope: { kind: "application" },
    kind: "workflow",
    state: "running",
    stateVersion: 1,
    parentRunId: null,
    retryOfRunId: null,
    forkOfRunId: null,
    waitReason: null,
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    stepAttempts: 1,
    expandedSteps: 1,
    outputBytes: 0,
    createdAt: 1,
    updatedAt: 1,
    completedAt: null,
  };
}
