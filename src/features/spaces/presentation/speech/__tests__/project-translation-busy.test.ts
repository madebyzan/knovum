import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../../foundation/i18n/i18n";
import type { SpeechProjectSnapshot } from "../../../contracts/generated/SpeechProjectSnapshot";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpeechProjectEditor from "../SpeechProjectEditor.vue";

it("prevents synthesis while the dubbing script is being translated and releases the editor afterward", async () => {
  const project: SpeechProjectSnapshot = {
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "video",
      projectType: "video_dubbing",
      title: "Dubbing",
      activeRevisionId: "revision",
      activeRevision: 1,
      segmentCount: 1,
      updatedAt: 1,
    },
    revision: {
      id: "revision",
      projectId: "project",
      revision: 1,
      sourceRevision: "source",
      pronunciationLexicon: {},
      createdAt: 1,
      outputConfig: {
        format: "mp4",
        video: { assetPath: "/video.mp4", sourceRevision: `sha256:${"a".repeat(64)}`, durationMs: 1000 },
        background: null,
        backgroundSeparation: null,
        voiceGainDb: 0,
        backgroundGainDb: -12,
        translation: null,
        preserveOriginalOutsideDialogue: false,
        sourceDialogueIntervals: [],
        retainOriginalTrack: true,
      },
    },
    segments: [
      {
        id: "line",
        ordinal: 0,
        text: "Hello",
        normalizedText: "Hello",
        voiceRevisionId: "voice",
        language: "en",
        state: "ready",
      },
    ],
  };
  const sourceNode: SpaceCanvasNode = {
    id: "video",
    type: "video",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "video", title: "Video", assetPath: "/video.mp4", fileName: "video.mp4" },
  };
  const wrapper = mount(SpeechProjectEditor, {
    props: {
      sourceNode,
      project,
      voices: [],
      running: false,
      submitting: false,
      saving: false,
      cancelling: false,
      previewListening: false,
    },
    global: {
      plugins: [i18n],
      stubs: { VideoDubbingEditor: true, VideoDeliveryExport: true, SpeechOutputReview: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  const editor = wrapper.getComponent({ name: "VideoDubbingEditor" });
  const generate = wrapper.get("button");
  expect(generate.attributes("disabled")).toBeUndefined();
  editor.vm.$emit("busy", true);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeDefined();
  expect(editor.props("disabled")).toBe(true);
  editor.vm.$emit("busy", false);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeUndefined();
  expect(editor.props("disabled")).toBe(false);
});
