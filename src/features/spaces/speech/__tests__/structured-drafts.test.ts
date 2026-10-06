import { expect, it } from "vitest";
import { effectScope, readonly } from "vue";

import type { AudiobookOutputConfig } from "../../contracts/generated/AudiobookOutputConfig";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { useAudiobookOutput } from "../use-audiobook-output";
import { useVideoDubbingDraft } from "../use-video-dubbing-draft";

it("loads an audiobook output from a Vue readonly value without sharing mutable metadata", () => {
  const config: AudiobookOutputConfig = {
    format: "m4b",
    sampleRate: 48000,
    loudnessLufs: -20,
    truePeakDb: -2,
    chapters: [],
    roles: [],
    paragraphPauseMs: 250,
    chapterPauseMs: 1000,
    wavPrecision: "pcm16",
    metadata: { title: "Original", author: "", narrator: "", cover: null },
  };
  const output = useAudiobookOutput();
  output.load(readonly(config));
  output.metadata.value.title = "Draft";
  expect(config.metadata.title).toBe("Original");
  expect(output.value().metadata.title).toBe("Draft");
});

it("loads video configuration as an isolated editable draft from a structured readonly response", () => {
  const config: VideoDubbingOutputConfig = {
    format: "mp4",
    video: { assetPath: "/video.mp4", sourceRevision: `sha256:${"a".repeat(64)}`, durationMs: 1000 },
    background: null,
    backgroundSeparation: null,
    translation: null,
    voiceGainDb: 0,
    backgroundGainDb: -12,
    preserveOriginalOutsideDialogue: false,
    sourceDialogueIntervals: [],
    retainOriginalTrack: false,
  };
  const project: SpeechProjectSnapshot = {
    project: {
      id: "project",
      spaceId: "space",
      projectType: "video_dubbing",
      title: "Video",
      activeRevisionId: "revision",
      activeRevision: 1,
      segmentCount: 0,
      updatedAt: 1,
    },
    revision: {
      id: "revision",
      projectId: "project",
      revision: 1,
      sourceRevision: "source",
      outputConfig: config,
      pronunciationLexicon: {},
      createdAt: 1,
    },
    segments: [],
  };
  const scope = effectScope();
  try {
    scope.run(() => {
      const draft = useVideoDubbingDraft(
        () => readonly(project),
        () => [],
      );
      expect(draft.dirty.value).toBe(false);
      draft.config.value!.video.assetPath = "/draft.mp4";
      expect(config.video.assetPath).toBe("/video.mp4");
      draft.setBackground({ ...config.video, assetPath: "/background.wav" });
      expect(draft.dirty.value).toBe(true);
      expect(config.background).toBeNull();
    });
  } finally {
    scope.stop();
  }
});
