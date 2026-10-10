import { expect, it } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";

import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useShortDubbingDraft } from "../use-short-dubbing-draft";

const project: SpeechProjectSnapshot = {
  project: {
    id: "project",
    spaceId: "space",
    sourceNodeId: "note",
    projectType: "short_dubbing",
    title: "Read aloud",
    activeRevisionId: "first",
    activeRevision: 1,
    segmentCount: 0,
    updatedAt: 1,
  },
  revision: {
    id: "first",
    projectId: "project",
    revision: 1,
    sourceRevision: "first",
    pronunciationLexicon: {},
    outputConfig: { format: "wav", wavPrecision: "pcm16" },
    createdAt: 1,
  },
  segments: [],
};
const voice: VoiceLibraryRevision = {
  profileId: "voice",
  revisionId: "voice-first",
  revision: 1,
  name: "Voice",
  sourceKind: "preset",
  engineId: "moss_nano",
  modelSpecId: "moss-tts-nano-onnx-v1",
  language: "zh-CN",
  supportsDeliveryInstruction: false,
  isActive: true,
  updatedAt: 1,
};
it("makes a new narration usable when its first voice is created without losing the text", async () => {
  const voices = shallowRef<VoiceLibraryRevision[]>([]);
  const scope = effectScope();
  try {
    const draft = scope.run(() =>
      useShortDubbingDraft(
        () => project,
        () => voices.value,
        () => "Notes to read",
      ),
    )!;
    expect(draft.valid.value).toBe(false);
    expect(draft.dirty.value).toBe(false);
    draft.text.value = "Edited notes";
    voices.value = [voice];
    await nextTick();
    expect(draft.voiceRevisionId.value).toBe(voice.revisionId);
    expect(draft.valid.value).toBe(true);
    expect(draft.dirty.value).toBe(true);
    expect(draft.revisionInput().segments[0]?.text).toBe("Edited notes");
    voices.value = [{ ...voice, revisionId: "second" }, voice];
    await nextTick();
    expect(draft.voiceRevisionId.value).toBe(voice.revisionId);
  } finally {
    scope.stop();
  }
});

it("persists a delivery instruction only for a voice version that supports it", async () => {
  const voices = shallowRef([{ ...voice, revisionId: "expressive", supportsDeliveryInstruction: true }]);
  const scope = effectScope();
  try {
    const draft = scope.run(() =>
      useShortDubbingDraft(
        () => project,
        () => voices.value,
        () => "Notes to read",
      ),
    )!;
    await nextTick();
    draft.deliveryInstruction.value = "Warm and measured";
    expect(draft.valid.value).toBe(true);
    expect(draft.revisionInput().segments[0]?.deliveryInstruction).toBe("Warm and measured");
    voices.value = [{ ...voice, revisionId: "expressive" }];
    await nextTick();
    expect(draft.deliveryInstruction.value).toBe("");
  } finally {
    scope.stop();
  }
});
