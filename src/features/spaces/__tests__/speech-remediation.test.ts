import { describe, expect, it } from "vitest";

import { defaultAiModelSettings } from "../../../foundation/ai/schema/model-settings";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { selectBackground, selectSeparatedBackground } from "../speech/video-background-selection";
import {
  applyTranslation,
  bindVideoSpeaker,
  translationInputs,
  type VideoLineDraft,
} from "../speech/video-dubbing-draft";
import { isLocalTranslationTarget, persistedTranslationTarget } from "../speech/video-translation-targets";
import { voiceRevisionOptions } from "../speech/voice-options";

function voice(id: string, revision: number, isActive: boolean, archivedAt?: number): VoiceLibraryRevision {
  const result: VoiceLibraryRevision = {
    profileId: `profile-${id}`,
    name: `Voice ${id}`,
    sourceKind: "preset",
    revisionId: id,
    revision,
    isActive,
    engineId: "mossNano",
    modelSpecId: "mossNano-v1",
    language: "en",
    supportsDeliveryInstruction: false,
    presetVoiceId: "0",
    updatedAt: 1,
  };
  if (archivedAt !== undefined) result.archivedAt = archivedAt;
  return result;
}

describe("speech architecture policies", () => {
  it("allows local-only translation only through loopback Ollama and restores exact provenance", () => {
    const settings = structuredClone(defaultAiModelSettings);
    settings.connections = [
      {
        id: "remote",
        name: "Remote",
        provider: "ollama",
        baseUrl: "https://models.example.com/v1",
        models: [{ id: "remote-model", name: "Remote", model: "qwen", capabilities: ["text-generation"] }],
      },
      {
        id: "local",
        name: "Local",
        provider: "ollama",
        baseUrl: "http://127.0.0.1:11434/v1",
        models: [{ id: "local-model", name: "Local", model: "qwen", capabilities: ["text-generation"] }],
      },
    ];
    const local = { type: "model" as const, connectionId: "local", modelId: "local-model" };
    const remote = { type: "model" as const, connectionId: "remote", modelId: "remote-model" };

    expect(isLocalTranslationTarget(settings, local)).toBe(true);
    expect(isLocalTranslationTarget(settings, remote)).toBe(false);
    expect(
      persistedTranslationTarget(settings, {
        runId: "translation-run",
        targetLanguage: "zh-CN",
        provider: "ollama",
        model: "qwen",
        localOnly: true,
      }),
    ).toEqual(local);
  });

  it("offers active voice revisions plus an already-bound archived revision", () => {
    const voices = [
      voice("active", 2, true),
      voice("historical", 1, false),
      voice("archived-bound", 1, true, 10),
      voice("archived-unused", 1, true, 10),
    ];

    expect(voiceRevisionOptions(voices, ["archived-bound"]).map(({ value }) => value)).toEqual([
      "active",
      "archived-bound",
    ]);
  });

  it("rejects stale translation results and keeps source segment revisions frozen per line", () => {
    const lines: VideoLineDraft[] = [
      {
        key: "draft-1",
        text: "Original",
        speakerKey: "",
        voiceRevisionId: "active",
        deliveryInstruction: "",
        startSeconds: "0",
        endSeconds: "1",
        needsTimingEdit: false,
        sourceSegmentId: "source-1",
        sourceSegmentRevision: 4,
      },
    ];
    const request = translationInputs(lines, false);
    expect(request).toEqual([{ sourceSegmentId: "source-1", sourceSegmentRevision: 4, text: "Original" }]);
    expect(() =>
      applyTranslation(
        lines,
        {
          provenance: {
            runId: "translation-run",
            targetLanguage: "zh-CN",
            provider: "ollama",
            model: "qwen",
            localOnly: true,
          },
          segments: [
            {
              stepId: "translation-step",
              sourceSegmentId: "stale-source",
              sourceSegmentRevision: 4,
              sourceText: "Original",
              translatedText: "译文",
            },
          ],
        },
        false,
      ),
    ).toThrow("does not match");
  });

  it("keeps per-line delivery instructions only when the selected voice supports them", () => {
    const line: VideoLineDraft = {
      key: "line",
      text: "Hello",
      speakerKey: "Narrator",
      voiceRevisionId: "expressive",
      deliveryInstruction: "Quiet and measured",
      startSeconds: "0",
      endSeconds: "1",
      needsTimingEdit: false,
    };
    expect(bindVideoSpeaker([line], "Narrator", "compact", false)[0]?.deliveryInstruction).toBe("");
    expect(bindVideoSpeaker([line], "Narrator", "expressive", true)[0]?.deliveryInstruction).toBe(
      "Quiet and measured",
    );
  });

  it("freezes accepted separation source identity and clears it for manual background selection", () => {
    const video = { assetPath: "/video.mp4", sourceRevision: `sha256:${"a".repeat(64)}`, durationMs: 1_000 };
    const background = {
      assetPath: "/background.wav",
      sourceRevision: `sha256:${"b".repeat(64)}`,
      durationMs: 1_000,
    };
    const config = {
      format: "mp4",
      video,
      background: null,
      backgroundSeparation: null,
      voiceGainDb: 0,
      backgroundGainDb: -12,
      translation: null,
      preserveOriginalOutsideDialogue: false,
      sourceDialogueIntervals: [{ startMs: 0, endMs: 1000 }],
      retainOriginalTrack: true,
    };
    const candidate: SpeechSourceSeparationCandidate = {
      separationId: "separation",
      spaceId: "space",
      runId: "separation",
      source: { ...video },
      background,
      vocals: { ...background, assetPath: "/managed/vocals.wav" },
      modelSpecId: "spleeter-2stems-fp16-cpu-v1",
      reviewStatus: "accepted",
      reviewWarning: "vocals_accompaniment_requires_full_audition",
    };
    const separated = selectSeparatedBackground(config, candidate);
    candidate.source.assetPath = "/changed.mp4";

    expect(separated.backgroundSeparation?.source).toEqual(video);
    expect(selectBackground(separated, background).backgroundSeparation).toBeNull();
  });
});
