import { describe, expect, it } from "vitest";

import { defaultAiModelSettings } from "../../../foundation/ai/schema/model-settings";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { VoiceProfileSummary } from "../speech/contracts";
import { selectBackground, selectSeparatedBackground } from "../speech/video-background-selection";
import { applyTranslation, translationInputs, type VideoLineDraft } from "../speech/video-dubbing-draft";
import { isLocalTranslationTarget, persistedTranslationTarget } from "../speech/video-translation-targets";
import { voiceRevisionOptions } from "../speech/voice-options";

function voice(id: string, revision: number, isActive: boolean, archivedAt?: number): VoiceProfileSummary {
  const result: VoiceProfileSummary = {
    id: `profile-${id}`,
    name: `Voice ${id}`,
    sourceKind: "preset",
    activeRevisionId: id,
    revision,
    isActive,
    engineId: "kokoro",
    modelSpecId: "kokoro-v1",
    language: "en",
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
      applyTranslation(lines, {
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
      }),
    ).toThrow("does not match");
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
    };
    const candidate: SpeechSourceSeparationCandidate = {
      separationId: "separation",
      spaceId: "space",
      runId: "separation",
      source: { ...video },
      background,
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
