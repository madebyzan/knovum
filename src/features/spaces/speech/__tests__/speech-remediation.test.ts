import { describe, expect, it } from "vitest";

import { DEFAULT_AI_MODEL_SETTINGS } from "../../../../foundation/ai/schema/model-settings";
import type { SpeechSourceSeparationPublication } from "../../contracts/generated/SpeechSourceSeparationPublication";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { selectBackground, selectSeparatedBackground } from "../video-background-selection";
import { applyTranslation, bindVideoSpeaker, type VideoLineDraft } from "../video-dubbing-draft";
import { persistedTranslationTarget } from "../video-translation-targets";
import { voiceRevisionOptions } from "../voice-options";

function voice(
  profileId: string,
  revision: number,
  isActive: boolean,
  archivedAt?: number,
): VoiceLibraryRevision {
  const voiceRevision: VoiceLibraryRevision = {
    profileId: `profile-${profileId}`,
    name: `Voice ${profileId}`,
    sourceKind: "preset",
    revisionId: profileId,
    revision,
    isActive,
    engineId: "mossNano",
    modelSpecId: "mossNano-v1",
    language: "en",
    supportsDeliveryInstruction: false,
    presetVoiceId: "0",
    updatedAt: 1,
  };
  if (archivedAt !== undefined) voiceRevision.archivedAt = archivedAt;
  return voiceRevision;
}

describe("speech architecture policies", () => {
  it("restores the model that produced a translation from its provenance", () => {
    const settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
    settings.connections = [
      {
        id: "openai",
        name: "OpenAI",
        provider: "openai",
        baseUrl: "https://api.openai.com/v1",
        models: [{ id: "gpt", name: "GPT", model: "qwen", capabilities: ["text-generation"] }],
      },
      {
        id: "local",
        name: "Local",
        provider: "ollama",
        baseUrl: "http://127.0.0.1:11434/v1",
        models: [{ id: "local-model", name: "Local", model: "qwen", capabilities: ["text-generation"] }],
      },
    ];

    expect(
      persistedTranslationTarget(settings, {
        runId: "translation-run",
        targetLanguage: "zh-CN",
        provider: "ollama",
        model: "qwen",
      }),
    ).toEqual({ type: "model", connectionId: "local", modelId: "local-model" });
    expect(
      persistedTranslationTarget(settings, {
        runId: "translation-run",
        targetLanguage: "zh-CN",
        provider: "anthropic",
        model: "qwen",
      }),
    ).toBeUndefined();
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
        sourceSegmentId: "source-1",
        sourceSegmentRevision: 4,
      },
    ];
    expect(() =>
      applyTranslation(lines, {
        provenance: {
          runId: "translation-run",
          targetLanguage: "zh-CN",
          provider: "ollama",
          model: "qwen",
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

  it("keeps per-line delivery instructions only when the selected voice supports them", () => {
    const line: VideoLineDraft = {
      key: "line",
      text: "Hello",
      speakerKey: "Narrator",
      voiceRevisionId: "expressive",
      deliveryInstruction: "Quiet and measured",
      startSeconds: "0",
      endSeconds: "1",
    };
    expect(bindVideoSpeaker([line], "Narrator", "compact", false)[0]?.deliveryInstruction).toBe("");
    expect(bindVideoSpeaker([line], "Narrator", "expressive", true)[0]?.deliveryInstruction).toBe(
      "Quiet and measured",
    );
  });

  it("freezes published separation source identity and clears it for manual background selection", () => {
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
    const publication: SpeechSourceSeparationPublication = {
      separationId: "separation",
      spaceId: "space",
      sourceNodeId: "source",
      runId: "separation",
      source: { ...video },
      background,
      vocals: { ...background, assetPath: "/managed/vocals.wav" },
      modelSpecId: "spleeter-2stems-fp16-cpu-v1",
    };
    const separated = selectSeparatedBackground(config, publication);
    publication.source.assetPath = "/changed.mp4";

    expect(separated.backgroundSeparation?.source).toEqual(video);
    expect(selectBackground(separated, background).backgroundSeparation).toBeNull();
  });
});
