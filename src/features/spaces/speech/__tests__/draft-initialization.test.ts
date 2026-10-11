import { expect, it, onTestFinished } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";

import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceMediaTranscript } from "../../media/schema/transcript";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { speechProjectInput } from "../speech-project-input";
import { useAudiobookDraft } from "../use-audiobook-draft";
import { useVideoDubbingDraft } from "../use-video-dubbing-draft";

const source: SpaceCanvasNode = {
  id: "source",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Book", text: "# Chapter\n\nRead this." },
};
const voice: VoiceLibraryRevision = {
  profileId: "voice",
  revisionId: "voice-first",
  revision: 1,
  name: "Voice",
  sourceKind: "preset",
  engineId: "moss_nano",
  modelSpecId: "moss-tts-nano-100m-onnx-native-v2",
  language: "zh-CN",
  supportsDeliveryInstruction: false,
  isActive: true,
  updatedAt: 1,
};
function project(projectType: "audiobook" | "video_dubbing"): SpeechProjectSnapshot {
  const input = speechProjectInput(source, projectType, {
    assetPath: "/video.mp4",
    sourceRevision: `sha256:${"a".repeat(64)}`,
    durationMs: 1000,
  })!;
  return {
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "source",
      projectType,
      title: "Book",
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
      outputConfig: input.outputConfig,
      createdAt: 1,
    },
    segments: [],
  };
}
const script = {
  schemaVersion: 1 as const,
  chapters: [{ title: "Chapter", paragraphs: [{ role: "Narrator", text: "Read this." }] }],
};
const transcript: SpaceMediaTranscript = {
  id: "transcript",
  sourceRevision: "video",
  revision: 1,
  language: "zh-CN",
  durationMs: 1000,
  text: "Hello",
  subtitleCues: [],
  speakerTracks: [],
  provenance: { engineId: "asr", modelSpecId: "model", modelRevision: "first", timing: "segment_only" },
  segments: [
    {
      id: "segment",
      revision: 1,
      text: "Hello",
      startMs: 0,
      endMs: 1000,
      startSample: 0,
      endSample: 48000,
      speakerId: null,
      units: [],
    },
  ],
};
it("opens an automatically imported book clean, but retains manual edits and imports as unsaved", async () => {
  const scope = effectScope();
  onTestFinished(() => scope.stop());
  const voices = shallowRef<VoiceLibraryRevision[]>([]);
  const draft = scope.run(() =>
    useAudiobookDraft(
      () => project("audiobook"),
      () => voices.value,
    ),
  )!;
  draft.importScript(script, true);
  expect(draft.dirty.value).toBe(false);
  expect(draft.needsRevision.value).toBe(true);
  expect(draft.valid.value).toBe(false);
  voices.value = [voice];
  await nextTick();
  expect(draft.dirty.value).toBe(false);
  expect(draft.valid.value).toBe(true);
  draft.output.metadata.value = { ...draft.output.metadata.value, author: "Author" };
  expect(draft.dirty.value).toBe(true);
  draft.output.metadata.value = { ...draft.output.metadata.value, author: "" };
  expect(draft.dirty.value).toBe(false);
  draft.importScript(script);
  expect(draft.dirty.value).toBe(true);
});
it("does not erase a book metadata edit while its first source import completes", () => {
  const scope = effectScope();
  onTestFinished(() => scope.stop());
  const draft = scope.run(() =>
    useAudiobookDraft(
      () => project("audiobook"),
      () => [],
    ),
  )!;
  draft.output.metadata.value = { ...draft.output.metadata.value, author: "Author" };
  draft.importScript(script, true);
  expect(draft.dirty.value).toBe(true);
});
it("does not require a leave decision for automatic transcripts or default voices", async () => {
  const scope = effectScope();
  onTestFinished(() => scope.stop());
  const voices = shallowRef<VoiceLibraryRevision[]>([]);
  const draft = scope.run(() =>
    useVideoDubbingDraft(
      () => project("video_dubbing"),
      () => voices.value,
    ),
  )!;
  draft.importTranscript(transcript, true);
  expect(draft.dirty.value).toBe(false);
  expect(draft.needsRevision.value).toBe(true);
  voices.value = [voice];
  await nextTick();
  expect(draft.dirty.value).toBe(false);
  expect(draft.valid.value).toBe(true);
  draft.lines.value[0]!.text = "Edited line";
  expect(draft.dirty.value).toBe(true);
});
