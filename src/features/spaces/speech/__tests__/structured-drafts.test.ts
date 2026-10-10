import { expect, it } from "vitest";
import { effectScope, readonly, shallowRef } from "vue";

import type { AudiobookOutputConfig } from "../../contracts/generated/AudiobookOutputConfig";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { useAudiobookOutput } from "../use-audiobook-output";
import { useShortDubbingDraft } from "../use-short-dubbing-draft";
import { useVideoDubbingDraft } from "../use-video-dubbing-draft";

it("seeds a new narration from its node and keeps saved revisions authoritative", async () => {
  const project = shallowRef<SpeechProjectSnapshot>({
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "note",
      projectType: "short_dubbing",
      title: "Narration",
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
  });
  const scope = effectScope();
  try {
    const draft = scope.run(() =>
      useShortDubbingDraft(
        () => project.value,
        () => [],
        () => "Node text",
      ),
    )!;
    expect(draft.text.value).toBe("Node text");
    expect(draft.dirty.value).toBe(false);
    expect(draft.needsRevision.value).toBe(true);
    draft.text.value = "Edited narration";
    expect(draft.dirty.value).toBe(true);
    draft.text.value = "Node text";
    expect(draft.dirty.value).toBe(false);
    project.value = {
      ...project.value,
      project: { ...project.value.project, activeRevision: 2, activeRevisionId: "second" },
      revision: { ...project.value.revision, id: "second", revision: 2 },
    };
    await Promise.resolve();
    expect(draft.text.value).toBe("");
    expect(draft.dirty.value).toBe(false);
  } finally {
    scope.stop();
  }
});

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
      sourceNodeId: "video",
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
